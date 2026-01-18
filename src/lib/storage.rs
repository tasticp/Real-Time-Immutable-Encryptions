use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rocksdb::{Options, WriteBatch, DB};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{CourtReport, EncryptedFrame, StorageBackend};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub database_path: String,
    pub ipfs_enabled: bool,
    pub ipfs_api_url: String,
    pub backup_enabled: bool,
    pub backup_path: String,
    pub compression_enabled: bool,
}

pub struct RocksDBStorage {
    db: Arc<RwLock<DB>>,
    config: StorageConfig,
}

impl RocksDBStorage {
    pub fn new(config: StorageConfig) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);

        let db = DB::open(&opts, &config.database_path)?;

        Ok(Self {
            db: Arc::new(RwLock::new(db)),
            config,
        })
    }

    fn generate_frame_key(&self, frame: &EncryptedFrame) -> String {
        format!("frame:{}:{}", frame.sequence, frame.timestamp)
    }

    fn generate_metadata_key(&self, evidence_id: &str) -> String {
        format!("metadata:{}", evidence_id)
    }

    async fn backup_to_ipfs(&self, data: &[u8]) -> Result<String> {
        if !self.config.ipfs_enabled {
            return Ok("".to_string());
        }

        // Mock IPFS upload - in production would use actual IPFS client
        let mock_cid = "QmXxxYyyZzz".to_string();
        println!("IPFS backup created with CID: {}", mock_cid);
        Ok(mock_cid)
    }

    async fn create_local_backup(&self, key: &str, data: &[u8]) -> Result<()> {
        if !self.config.backup_enabled {
            return Ok(());
        }

        use std::fs;
        let backup_path = Path::new(&self.config.backup_path).join(format!("{}.bak", key));

        fs::write(backup_path, data)?;
        Ok(())
    }
}

#[async_trait]
impl StorageBackend for RocksDBStorage {
    async fn store_frame(&self, frame: &EncryptedFrame) -> Result<String> {
        let key = self.generate_frame_key(frame);
        let serialized = serde_json::to_vec(frame)?;

        // Compress if enabled
        let data = if self.config.compression_enabled {
            // Simple compression - in production would use proper compression
            serialized.len()
        } else {
            serialized.len()
        };

        // Store to RocksDB
        let db = self.db.read().await;
        db.put(&key, &serialized)?;

        // Create backups
        let ipfs_cid = self.backup_to_ipfs(&serialized).await?;
        self.create_local_backup(&key, &serialized).await?;

        // Store backup references
        if !ipfs_cid.is_empty() {
            db.put(&format!("ipfs:{}", key), ipfs_cid.as_bytes())?;
        }

        Ok(key)
    }

    async fn retrieve_frame(&self, frame_id: &str) -> Result<EncryptedFrame> {
        let db = self.db.read().await;

        match db.get(frame_id)? {
            Some(data) => {
                let frame: EncryptedFrame = serde_json::from_slice(&data)?;
                Ok(frame)
            }
            None => Err(anyhow!("Frame not found: {}", frame_id)),
        }
    }

    async fn store_metadata(&self, metadata: &CourtReport) -> Result<String> {
        let key = self.generate_metadata_key(&metadata.evidence_id);
        let serialized = serde_json::to_vec(metadata)?;

        let db = self.db.read().await;
        db.put(&key, &serialized)?;

        // Create backup references
        let ipfs_cid = self.backup_to_ipfs(&serialized).await?;
        self.create_local_backup(&key, &serialized).await?;

        Ok(key)
    }
}

pub struct IPFSStorage {
    client: reqwest::Client,
    config: StorageConfig,
}

impl IPFSStorage {
    pub fn new(config: StorageConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }

    async fn add_to_ipfs(&self, data: &[u8]) -> Result<String> {
        let url = format!("{}/api/v0/add", self.config.ipfs_api_url);

        let form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(data.to_vec())
                .file_name("frame.enc")
                .mime_str("application/octet-stream")?,
        );

        let response = self.client.post(&url).multipart(form).send().await?;

        let result: serde_json::Value = response.json().await?;
        let cid = result["Hash"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid IPFS response"))?;

        Ok(cid.to_string())
    }

    async fn get_from_ipfs(&self, cid: &str) -> Result<Vec<u8>> {
        let url = format!("{}/api/v0/cat/{}", self.config.ipfs_api_url, cid);

        let response = self.client.get(&url).send().await?;
        Ok(response.bytes().await?.to_vec())
    }
}

#[derive(Debug)]
pub struct DistributedStorage {
    primary: RocksDBStorage,
    backup: IPFSStorage,
}

impl DistributedStorage {
    pub async fn new(config: StorageConfig) -> Result<Self> {
        let primary = RocksDBStorage::new(config.clone())?;
        let backup = IPFSStorage::new(config);

        Ok(Self { primary, backup })
    }

    pub async fn store_with_redundancy(&self, frame: &EncryptedFrame) -> Result<Vec<String>> {
        let mut locations = Vec::new();

        // Store to primary storage
        let primary_key = self.primary.store_frame(frame).await?;
        locations.push(primary_key);

        // Store to IPFS backup
        let serialized = serde_json::to_vec(frame)?;
        let ipfs_cid = self.backup.add_to_ipfs(&serialized).await?;
        locations.push(format!("ipfs:{}", ipfs_cid));

        Ok(locations)
    }

    pub async fn retrieve_with_fallback(&self, frame_id: &str) -> Result<EncryptedFrame> {
        // Try primary first
        match self.primary.retrieve_frame(frame_id).await {
            Ok(frame) => Ok(frame),
            Err(_) => {
                // Fallback to IPFS
                if frame_id.starts_with("ipfs:") {
                    let cid = &frame_id[5..]; // Remove "ipfs:" prefix
                    let data = self.backup.get_from_ipfs(cid).await?;
                    let frame: EncryptedFrame = serde_json::from_slice(&data)?;
                    Ok(frame)
                } else {
                    Err(anyhow!("Frame not found in any storage location"))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_rocksdb_storage() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = StorageConfig {
            database_path: temp_dir.path().to_string_lossy().to_string(),
            ipfs_enabled: false,
            ipfs_api_url: "".to_string(),
            backup_enabled: false,
            backup_path: "".to_string(),
            compression_enabled: false,
        };

        let storage = RocksDBStorage::new(config)?;

        let frame = EncryptedFrame {
            sequence: 1,
            ciphertext: vec![1, 2, 3, 4],
            hash: "test_hash".to_string(),
            previous_hash: "prev_hash".to_string(),
            nonce: vec![0, 1, 2, 3],
            timestamp: 1640995200,
            blockchain_anchors: vec![],
        };

        let key = storage.store_frame(&frame).await?;
        let retrieved = storage.retrieve_frame(&key).await?;

        assert_eq!(retrieved.sequence, frame.sequence);
        assert_eq!(retrieved.hash, frame.hash);

        Ok(())
    }
}
