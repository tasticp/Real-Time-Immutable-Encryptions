use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImmutableEncryptionError {
    #[error("Cryptography error: {0}")]
    Crypto(String),

    #[error("Blockchain error: {0}")]
    Blockchain(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Verification error: {0}")]
    Verification(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Hardware security error: {0}")]
    Hardware(String),

    #[error("Video processing error: {0}")]
    Video(String),

    #[error("Invalid frame sequence: {0}")]
    InvalidSequence(u64),

    #[error("Hash chain integrity violation")]
    HashChainViolation,

    #[error("Insufficient blockchain confirmations: {chain} requires {required}, got {actual}")]
    InsufficientConfirmations {
        chain: String,
        required: u64,
        actual: u64,
    },

    #[error("Frame not found: {frame_id}")]
    FrameNotFound { frame_id: String },

    #[error("Quantum cryptography not available")]
    QuantumCryptoUnavailable,

    #[error("Hardware attestation failed: {0}")]
    AttestationFailed(String),

    #[error("Evidence tampered: {details}")]
    EvidenceTampered { details: String },

    #[error("Legal compliance check failed: {0}")]
    LegalComplianceFailed(String),

    #[error("Insufficient permissions: {0}")]
    PermissionDenied(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Resource temporarily unavailable: {0}")]
    ResourceUnavailable(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl ImmutableEncryptionError {
    pub fn crypto(msg: &str) -> Self {
        Self::Crypto(msg.to_string())
    }

    pub fn blockchain(msg: &str) -> Self {
        Self::Blockchain(msg.to_string())
    }

    pub fn storage(msg: &str) -> Self {
        Self::Storage(msg.to_string())
    }

    pub fn verification(msg: &str) -> Self {
        Self::Verification(msg.to_string())
    }

    pub fn config(msg: &str) -> Self {
        Self::Config(msg.to_string())
    }

    pub fn network(msg: &str) -> Self {
        Self::Network(msg.to_string())
    }

    pub fn hardware(msg: &str) -> Self {
        Self::Hardware(msg.to_string())
    }

    pub fn video(msg: &str) -> Self {
        Self::Video(msg.to_string())
    }

    pub fn internal(msg: &str) -> Self {
        Self::Internal(msg.to_string())
    }
}

// Conversion from common error types
impl From<std::io::Error> for ImmutableEncryptionError {
    fn from(err: std::io::Error) -> Self {
        Self::Storage(err.to_string())
    }
}

impl From<serde_json::Error> for ImmutableEncryptionError {
    fn from(err: serde_json::Error) -> Self {
        Self::Verification(format!("JSON serialization error: {}", err))
    }
}

impl From<toml::de::Error> for ImmutableEncryptionError {
    fn from(err: toml::de::Error) -> Self {
        Self::Config(format!("TOML parsing error: {}", err))
    }
}

impl From<reqwest::Error> for ImmutableEncryptionError {
    fn from(err: reqwest::Error) -> Self {
        Self::Network(format!("HTTP request error: {}", err))
    }
}
