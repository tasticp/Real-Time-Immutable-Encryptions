use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub encryption: EncryptionConfig,
    pub blockchain: BlockchainConfig,
    pub storage: StorageConfig,
    pub verification: VerificationConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub request_timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub primary_key_path: String,
    pub key_rotation_interval_seconds: u64,
    pub quantum_resistant: bool,
    pub hardware_backed: bool,
    pub compression_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub ethereum: EthereumConfig,
    pub bitcoin: BitcoinConfig,
    pub private_chain: PrivateChainConfig,
    pub opentimestamps: OpenTimestampsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumConfig {
    pub rpc_url: String,
    pub contract_address: Option<String>,
    pub gas_limit: u64,
    pub gas_price_gwei: f64,
    pub confirmations_required: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinConfig {
    pub rpc_url: String,
    pub wallet_name: String,
    pub fee_sat_per_byte: u64,
    pub confirmations_required: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateChainConfig {
    pub rpc_url: String,
    pub organization_id: String,
    pub consensus_mechanism: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenTimestampsConfig {
    pub enabled: bool,
    pub calendar_urls: Vec<String>,
    pub fallback_calendars: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub database_path: String,
    pub ipfs: IPFSConfig,
    pub backup: BackupConfig,
    pub retention_days: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPFSConfig {
    pub enabled: bool,
    pub api_url: String,
    pub gateway_url: String,
    pub pin_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub enabled: bool,
    pub backup_path: String,
    pub backup_interval_hours: u64,
    pub max_backups: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationConfig {
    pub strict_mode: bool,
    pub quantum_verification: bool,
    pub hardware_attestation: bool,
    pub min_confirmations: HashMap<String, u64>,
    pub evidence_retention_years: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub max_file_size_mb: u64,
    pub max_files: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                max_connections: 1000,
                request_timeout_ms: 30000,
            },
            encryption: EncryptionConfig {
                primary_key_path: "keys/primary.key".to_string(),
                key_rotation_interval_seconds: 3600,
                quantum_resistant: true,
                hardware_backed: true,
                compression_enabled: true,
            },
            blockchain: BlockchainConfig {
                ethereum: EthereumConfig {
                    rpc_url: "https://mainnet.infura.io/v3/YOUR_PROJECT_ID".to_string(),
                    contract_address: None,
                    gas_limit: 100000,
                    gas_price_gwei: 20.0,
                    confirmations_required: 12,
                },
                bitcoin: BitcoinConfig {
                    rpc_url: "https://blockstream.info/api".to_string(),
                    wallet_name: "evidence_wallet".to_string(),
                    fee_sat_per_byte: 10,
                    confirmations_required: 6,
                },
                private_chain: PrivateChainConfig {
                    rpc_url: "http://localhost:8545".to_string(),
                    organization_id: "your_org".to_string(),
                    consensus_mechanism: "raft".to_string(),
                },
                opentimestamps: OpenTimestampsConfig {
                    enabled: true,
                    calendar_urls: vec![
                        "https://a.calendar.opentimestamps.org".to_string(),
                        "https://b.calendar.opentimestamps.org".to_string(),
                    ],
                    fallback_calendars: vec![
                        "https://alice.btc.calendar.opentimestamps.org".to_string(),
                        "https://bob.btc.calendar.opentimestamps.org".to_string(),
                    ],
                },
            },
            storage: StorageConfig {
                database_path: "data/blockchain.db".to_string(),
                ipfs: IPFSConfig {
                    enabled: true,
                    api_url: "http://localhost:5001".to_string(),
                    gateway_url: "http://localhost:8080".to_string(),
                    pin_enabled: true,
                },
                backup: BackupConfig {
                    enabled: true,
                    backup_path: "backups".to_string(),
                    backup_interval_hours: 24,
                    max_backups: 30,
                },
                retention_days: 365 * 7, // 7 years
            },
            verification: VerificationConfig {
                strict_mode: true,
                quantum_verification: true,
                hardware_attestation: true,
                min_confirmations: {
                    let mut map = HashMap::new();
                    map.insert("bitcoin".to_string(), 6u64);
                    map.insert("ethereum".to_string(), 12u64);
                    map.insert("private".to_string(), 3u64);
                    map
                },
                evidence_retention_years: 10,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: Some("logs/immutable_encryption.log".to_string()),
                max_file_size_mb: 100,
                max_files: 10,
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        // Try to load from environment variables first
        if let Ok(config_path) = std::env::var("CONFIG_PATH") {
            Self::load_from_file(&config_path)
        } else if std::path::Path::new("config.toml").exists() {
            Self::load_from_file("config.toml")
        } else {
            tracing::info!("Using default configuration");
            Ok(Self::default())
        }
    }

    pub fn load_from_file(path: &str) -> Result<Self> {
        let config_content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&config_content)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let config_content = toml::to_string_pretty(self)?;
        std::fs::write(path, config_content)?;
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        // Validate server config
        if self.server.port == 0 {
            return Err(anyhow!("Server port cannot be 0"));
        }

        // Validate encryption config
        if self.encryption.primary_key_path.is_empty() {
            return Err(anyhow!("Primary key path cannot be empty"));
        }

        // Validate blockchain configs
        if self.blockchain.ethereum.rpc_url.is_empty() {
            return Err(anyhow!("Ethereum RPC URL cannot be empty"));
        }

        if self.blockchain.bitcoin.rpc_url.is_empty() {
            return Err(anyhow!("Bitcoin RPC URL cannot be empty"));
        }

        // Validate storage config
        if self.storage.database_path.is_empty() {
            return Err(anyhow!("Database path cannot be empty"));
        }

        Ok(())
    }

    pub fn get_crypto_config(&self) -> crate::crypto::CryptoConfig {
        crate::crypto::CryptoConfig {
            primary_key: vec![0u8; 32], // Would load from file
            key_rotation_interval: self.encryption.key_rotation_interval_seconds,
            quantum_resistant: self.encryption.quantum_resistant,
            hardware_backed: self.encryption.hardware_backed,
        }
    }

    pub fn get_blockchain_config(&self) -> crate::blockchain::BlockchainConfig {
        crate::blockchain::BlockchainConfig {
            ethereum_rpc_url: self.blockchain.ethereum.rpc_url.clone(),
            bitcoin_rpc_url: self.blockchain.bitcoin.rpc_url.clone(),
            private_chain_rpc: self.blockchain.private_chain.rpc_url.clone(),
            opentimestamps_url: self
                .blockchain
                .opentimestamps
                .calendar_urls
                .first()
                .cloned()
                .unwrap_or_default(),
        }
    }

    pub fn get_storage_config(&self) -> crate::storage::StorageConfig {
        crate::storage::StorageConfig {
            database_path: self.storage.database_path.clone(),
            ipfs_enabled: self.storage.ipfs.enabled,
            ipfs_api_url: self.storage.ipfs.api_url.clone(),
            backup_enabled: self.storage.backup.enabled,
            backup_path: self.storage.backup.backup_path.clone(),
            compression_enabled: self.encryption.compression_enabled,
        }
    }

    pub fn get_verification_config(&self) -> crate::verification::VerificationConfig {
        crate::verification::VerificationConfig {
            strict_mode: self.verification.strict_mode,
            quantum_verification: self.verification.quantum_verification,
            hardware_attestation: self.verification.hardware_attestation,
            min_confirmations: self.verification.min_confirmations.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();

        assert_eq!(config.server.port, deserialized.server.port);
        assert_eq!(
            config.encryption.quantum_resistant,
            deserialized.encryption.quantum_resistant
        );
    }
}
