use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::{interval, Duration};

use crate::{
    blockchain::{BlockchainConfig, MultiChainAnchor},
    crypto::CryptoConfig,
    storage::{DistributedStorage, StorageConfig},
    verification::{VerificationConfig, VerificationEngine as Verifier},
    BlockchainAnchor, EncryptedFrame, EncryptionEngine, FrameMetadata, StorageBackend,
    VerificationEngine, VideoFrame,
};

#[derive(Debug)]
pub struct RealTimeEncryptionNode {
    encryption_engine: Arc<Mutex<EncryptionEngine>>,
    blockchain_anchor: Arc<MultiChainAnchor>,
    storage: Arc<DistributedStorage>,
    verifier: Arc<Verifier>,
    frame_buffer: Arc<RwLock<Vec<EncryptedFrame>>>,
}

impl RealTimeEncryptionNode {
    pub async fn new(
        crypto_config: CryptoConfig,
        blockchain_config: BlockchainConfig,
        storage_config: StorageConfig,
        verification_config: VerificationConfig,
    ) -> Result<Self> {
        let encryption_engine = Arc::new(Mutex::new(EncryptionEngine::new(crypto_config)?));

        let blockchain_anchor = Arc::new(MultiChainAnchor::new(blockchain_config).await?);

        let storage = Arc::new(DistributedStorage::new(storage_config).await?);

        let verifier = Arc::new(Verifier::new(verification_config));

        Ok(Self {
            encryption_engine,
            blockchain_anchor,
            storage,
            verifier,
            frame_buffer: Arc::new(RwLock::new(Vec::new())),
        })
    }

    pub async fn start_processing(&self) -> Result<(FrameSender, EncryptedFrameReceiver)> {
        let (tx, rx) = mpsc::unbounded_channel::<VideoFrame>();
        let (enc_tx, enc_rx) = mpsc::unbounded_channel::<EncryptedFrame>();

        // Start encryption pipeline
        let node = self.clone();
        tokio::spawn(async move {
            node.encryption_pipeline(tx, enc_tx).await;
        });

        // Start blockchain anchoring
        let node = self.clone();
        tokio::spawn(async move {
            node.blockchain_pipeline(enc_rx).await;
        });

        Ok((tx, self.create_verification_receiver().await))
    }

    async fn encryption_pipeline(&self, mut frame_rx: FrameReceiver, enc_tx: EncryptedFrameSender) {
        while let Some(frame) = frame_rx.recv().await {
            match self.process_frame(frame).await {
                Ok(encrypted_frame) => {
                    if let Err(e) = enc_tx.send(encrypted_frame) {
                        tracing::error!("Failed to send encrypted frame: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to process frame: {}", e);
                }
            }
        }
    }

    async fn blockchain_pipeline(&self, mut encrypted_rx: EncryptedFrameReceiver) {
        // Buffer frames for batch processing
        let mut buffer = Vec::new();
        let mut ticker = interval(Duration::from_secs(5)); // Process every 5 seconds

        loop {
            tokio::select! {
                frame = encrypted_rx.recv() => {
                    match frame {
                        Some(frame) => buffer.push(frame),
                        None => break, // Channel closed
                    }
                }
                _ = ticker.tick() => {
                    if !buffer.is_empty() {
                        if let Err(e) = self.process_frame_batch(&mut buffer).await {
                            tracing::error!("Failed to process frame batch: {}", e);
                        }
                    }
                }
            }
        }

        // Process remaining frames
        if !buffer.is_empty() {
            let _ = self.process_frame_batch(&mut buffer).await;
        }
    }

    async fn process_frame(&self, frame: VideoFrame) -> Result<EncryptedFrame> {
        let mut engine = self.encryption_engine.lock().await;

        // Generate frame hash
        let frame_hash = engine.generate_frame_hash(&frame)?;

        // Get previous hash from buffer
        let previous_hash = {
            let buffer = self.frame_buffer.read().await;
            buffer
                .last()
                .map(|f| f.hash.clone())
                .unwrap_or_else(|| "0".repeat(64))
        };

        // Create hash chain link
        let chain_hash =
            engine.create_hash_chain_link(&frame_hash, &previous_hash, frame.sequence)?;

        // Encrypt frame data
        let (ciphertext, nonce) = engine.encrypt_data(&frame.data, frame.timestamp)?;

        let encrypted_frame = EncryptedFrame {
            sequence: frame.sequence,
            ciphertext,
            hash: chain_hash,
            previous_hash,
            nonce,
            timestamp: frame.timestamp,
            blockchain_anchors: Vec::new(), // Will be filled in batch processing
        };

        // Add to buffer
        self.frame_buffer
            .write()
            .await
            .push(encrypted_frame.clone());

        Ok(encrypted_frame)
    }

    async fn process_frame_batch(&self, frames: &mut Vec<EncryptedFrame>) -> Result<()> {
        if frames.is_empty() {
            return Ok(());
        }

        // Sort frames by sequence to ensure proper order
        frames.sort_by_key(|f| f.sequence);

        // Process frames in parallel for blockchain anchoring
        let mut anchor_tasks = Vec::new();

        for frame in frames.iter() {
            let blockchain = self.blockchain_anchor.clone();
            let metadata = self.create_mock_metadata(frame.sequence);

            let task = tokio::spawn(async move {
                let hash = frame.hash.clone();
                blockchain.anchor_to_all_chains(&hash, &metadata).await
            });

            anchor_tasks.push(task);
        }

        // Wait for all blockchain anchors
        let anchor_results = futures::future::join_all(anchor_tasks).await;

        // Assign anchors to frames
        for (i, result) in anchor_results.into_iter().enumerate() {
            match result {
                Ok(Ok(anchors)) => {
                    if i < frames.len() {
                        frames[i].blockchain_anchors = anchors;
                    }
                }
                Ok(Err(e)) => {
                    tracing::error!("Failed to anchor frame {}: {}", frames[i].sequence, e);
                }
                Err(e) => {
                    tracing::error!("Blockchain anchoring task failed: {}", e);
                }
            }
        }

        // Store frames with redundancy
        let mut storage_tasks = Vec::new();

        for frame in frames.iter() {
            let storage = self.storage.clone();
            let frame_clone = frame.clone();

            let task =
                tokio::spawn(async move { storage.store_with_redundancy(&frame_clone).await });

            storage_tasks.push(task);
        }

        // Wait for all storage operations
        let storage_results = futures::future::join_all(storage_tasks).await;

        for (i, result) in storage_results.into_iter().enumerate() {
            match result {
                Ok(Ok(locations)) => {
                    tracing::info!("Frame {} stored at {:?}", frames[i].sequence, locations);
                }
                Ok(Err(e)) => {
                    tracing::error!("Failed to store frame {}: {}", frames[i].sequence, e);
                }
                Err(e) => {
                    tracing::error!("Storage task failed: {}", e);
                }
            }
        }

        // Clear processed frames
        frames.clear();

        Ok(())
    }

    fn create_mock_metadata(&self, sequence: u64) -> FrameMetadata {
        FrameMetadata {
            device_id: format!("device_{}", sequence % 3),
            location: Some((40.7128, -74.0060)), // Mock location
            resolution: (1920, 1080),
            fps: 30,
            codec: "H.264".to_string(),
        }
    }

    async fn create_verification_receiver(&self) -> EncryptedFrameReceiver {
        let (tx, rx) = mpsc::unbounded_channel();

        // This would be used for external verification requests
        // For now, we'll just return the receiver
        rx
    }

    pub async fn verify_evidence(&self, frame_ids: &[String]) -> Result<crate::VerificationResult> {
        let mut frames = Vec::new();

        // Retrieve frames
        for frame_id in frame_ids {
            match self.storage.retrieve_with_fallback(frame_id).await {
                Ok(frame) => frames.push(frame),
                Err(e) => tracing::error!("Failed to retrieve frame {}: {}", frame_id, e),
            }
        }

        if frames.is_empty() {
            return Err(anyhow!("No valid frames found for verification"));
        }

        // Sort by sequence
        frames.sort_by_key(|f| f.sequence);

        // Perform verification
        self.verifier.verify_integrity(&frames).await
    }

    pub async fn generate_court_report(&self, evidence_id: &str) -> Result<crate::CourtReport> {
        // In a real implementation, would retrieve all frames for the evidence
        let mock_frames = Vec::new(); // Would be populated from storage
        self.verifier
            .generate_court_report(evidence_id.to_string(), &mock_frames)
    }
}

impl Clone for RealTimeEncryptionNode {
    fn clone(&self) -> Self {
        Self {
            encryption_engine: self.encryption_engine.clone(),
            blockchain_anchor: self.blockchain_anchor.clone(),
            storage: self.storage.clone(),
            verifier: self.verifier.clone(),
            frame_buffer: self.frame_buffer.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_node_initialization() -> Result<()> {
        let temp_dir = TempDir::new()?;

        let crypto_config = CryptoConfig {
            primary_key: vec![0u8; 32],
            key_rotation_interval: 60,
            quantum_resistant: false,
            hardware_backed: false,
        };

        let blockchain_config = BlockchainConfig {
            ethereum_rpc_url: "https://mainnet.infura.io/v3/test".to_string(),
            bitcoin_rpc_url: "https://blockstream.info/api".to_string(),
            private_chain_rpc: "http://localhost:8545".to_string(),
            opentimestamps_url: "https://ots.btc.catallaxy.com".to_string(),
        };

        let storage_config = StorageConfig {
            database_path: temp_dir.path().to_string_lossy().to_string(),
            ipfs_enabled: false,
            ipfs_api_url: "".to_string(),
            backup_enabled: false,
            backup_path: "".to_string(),
            compression_enabled: false,
        };

        let verification_config = VerificationConfig {
            strict_mode: true,
            quantum_verification: false,
            hardware_attestation: false,
            min_confirmations: HashMap::new(),
        };

        let node = RealTimeEncryptionNode::new(
            crypto_config,
            blockchain_config,
            storage_config,
            verification_config,
        )
        .await?;

        assert!(true); // Node created successfully

        Ok(())
    }
}
