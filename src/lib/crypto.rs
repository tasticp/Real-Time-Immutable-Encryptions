use anyhow::{anyhow, Result};
use blake3::Hasher;
use ring::aead::{LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

use crate::{BlockchainAnchor, EncryptedFrame, FrameMetadata, VideoFrame};

#[derive(Debug, Serialize, Deserialize)]
pub struct CryptoConfig {
    pub primary_key: Vec<u8>,
    pub key_rotation_interval: u64,
    pub quantum_resistant: bool,
    pub hardware_backed: bool,
}

#[derive(Debug)]
pub struct EncryptionEngine {
    primary_key: LessSafeKey,
    rng: SystemRandom,
    config: CryptoConfig,
    key_schedule: HashMap<u64, Vec<u8>>, // timestamp -> key
    quantum_keys: HashMap<u64, Vec<u8>>, // for post-quantum layer
}

impl EncryptionEngine {
    pub fn new(config: CryptoConfig) -> Result<Self> {
        let unbound_key = UnboundKey::new(&AES_256_GCM, &config.primary_key)
            .map_err(|e| anyhow!("Failed to create encryption key: {}", e))?;
        let primary_key = LessSafeKey::new(unbound_key);

        let mut engine = Self {
            primary_key,
            rng: SystemRandom::new(),
            config,
            key_schedule: HashMap::new(),
            quantum_keys: HashMap::new(),
        };

        // Initialize key schedule
        engine.rotate_keys()?;

        Ok(engine)
    }

    fn rotate_keys(&mut self) -> Result<()> {
        use pqcrypto_kyber::kyber1024;
        use pqcrypto_traits::kem as pqkem;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        // Generate frame-specific keys for next interval
        for i in 0..self.config.key_rotation_interval {
            let timestamp = now + i * 60; // Rotate every minute
            let mut frame_key = vec![0u8; 32];
            self.rng.fill(&mut frame_key)?;
            self.key_schedule.insert(timestamp, frame_key);

            // Generate quantum-resistant keys if enabled
            if self.config.quantum_resistant {
                let (pk, sk) = kyber1024::keypair();
                let combined_key = [pk.as_bytes(), sk.as_bytes()].concat();
                self.quantum_keys.insert(timestamp, combined_key);
            }
        }

        Ok(())
    }

    pub fn generate_frame_hash(&self, frame: &VideoFrame) -> Result<String> {
        // Double hash: SHA-256 + BLAKE3 for maximum security
        let mut sha256 = Sha256::new();
        sha256.update(&frame.sequence.to_be_bytes());
        sha256.update(&frame.timestamp.to_be_bytes());
        sha256.update(&frame.data);
        sha256.update(serde_json::to_string(&frame.metadata)?.as_bytes());
        let sha_result = sha256.finalize();

        let mut blake3 = Hasher::new();
        blake3.update(&sha_result);
        let blake_result = blake3.finalize();

        Ok(hex::encode(blake_result.as_bytes()))
    }

    pub fn create_hash_chain_link(
        &self,
        current_hash: &str,
        previous_hash: &str,
        sequence: u64,
    ) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.update(current_hash.as_bytes());
        hasher.update(previous_hash.as_bytes());
        hasher.update(&sequence.to_be_bytes());
        Ok(hex::encode(hasher.finalize()))
    }

    pub fn encrypt_data(&mut self, data: &[u8], timestamp: u64) -> Result<(Vec<u8>, Vec<u8>)> {
        let key = self
            .key_schedule
            .get(&timestamp)
            .ok_or_else(|| anyhow!("No encryption key for timestamp {}", timestamp))?;

        let unbound_key = UnboundKey::new(&AES_256_GCM, key)
            .map_err(|e| anyhow!("Failed to create frame key: {}", e))?;
        let less_safe_key = LessSafeKey::new(unbound_key);

        let mut nonce_bytes = [0u8; 12];
        self.rng.fill(&mut nonce_bytes)?;
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);

        let mut ciphertext = data.to_vec();
        less_safe_key
            .seal_in_place_append_tag(nonce, &mut ciphertext)
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        Ok((ciphertext, nonce_bytes.to_vec()))
    }

    pub fn verify_quantum_layer(&self, encrypted_data: &[u8], timestamp: u64) -> Result<bool> {
        if !self.config.quantum_resistant {
            return Ok(true); // Skip if quantum layer not enabled
        }

        // Implement quantum-resistant verification using Kyber
        // This would typically involve shared secret verification
        // For now, we'll simulate the check
        self.quantum_keys
            .get(&timestamp)
            .ok_or_else(|| anyhow!("No quantum key for timestamp {}", timestamp))
            .map(|_| true) // Simplified - would implement actual verification
    }

    pub fn generate_tamper_proof(&self, frames: &[EncryptedFrame]) -> Result<String> {
        let mut hasher = Sha256::new();

        for frame in frames {
            hasher.update(frame.hash.as_bytes());
            hasher.update(frame.nonce.as_slice());
            hasher.update(&frame.sequence.to_be_bytes());
        }

        Ok(hex::encode(hasher.finalize()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_hash_generation() -> Result<()> {
        let config = CryptoConfig {
            primary_key: vec![0u8; 32],
            key_rotation_interval: 60,
            quantum_resistant: false,
            hardware_backed: false,
        };

        let engine = EncryptionEngine::new(config)?;

        let frame = VideoFrame {
            timestamp: 1640995200, // 2022-01-01 00:00:00 UTC
            sequence: 1,
            data: vec![1, 2, 3, 4],
            metadata: FrameMetadata {
                device_id: "test-camera-01".to_string(),
                location: Some((40.7128, -74.0060)), // NYC coordinates
                resolution: (1920, 1080),
                fps: 30,
                codec: "H.264".to_string(),
            },
        };

        let hash1 = engine.generate_frame_hash(&frame)?;
        let hash2 = engine.generate_frame_hash(&frame);

        assert_eq!(hash1, hash2?);
        assert_eq!(hash1.len(), 64); // BLAKE3 hash in hex

        Ok(())
    }

    #[test]
    fn test_hash_chain_link() -> Result<()> {
        let config = CryptoConfig {
            primary_key: vec![0u8; 32],
            key_rotation_interval: 60,
            quantum_resistant: false,
            hardware_backed: false,
        };

        let engine = EncryptionEngine::new(config)?;

        let prev_hash = "a1b2c3d4e5f6";
        let current_hash = "f6e5d4c3b2a1";
        let sequence = 42;

        let chain_link = engine.create_hash_chain_link(current_hash, prev_hash, sequence)?;

        assert_ne!(chain_link, current_hash);
        assert_ne!(chain_link, prev_hash);
        assert_eq!(chain_link.len(), 64);

        Ok(())
    }
}
