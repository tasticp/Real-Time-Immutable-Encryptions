pub mod blockchain;
pub mod config;
pub mod crypto;
pub mod error;
pub mod storage;
pub mod verification;
pub mod video;

use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub timestamp: u64,
    pub sequence: u64,
    pub data: Vec<u8>,
    pub metadata: FrameMetadata,
}

#[derive(Debug, Clone)]
pub struct FrameMetadata {
    pub device_id: String,
    pub location: Option<(f64, f64)>,
    pub resolution: (u32, u32),
    pub fps: u32,
    pub codec: String,
}

#[derive(Debug, Clone)]
pub struct EncryptedFrame {
    pub sequence: u64,
    pub ciphertext: Vec<u8>,
    pub hash: String,
    pub previous_hash: String,
    pub nonce: Vec<u8>,
    pub timestamp: u64,
    pub blockchain_anchors: Vec<BlockchainAnchor>,
}

#[derive(Debug, Clone)]
pub struct BlockchainAnchor {
    pub chain: String,
    pub transaction_hash: String,
    pub block_number: u64,
    pub timestamp: u64,
    pub proof: String,
}

#[derive(Debug)]
pub struct VerificationResult {
    pub is_valid: bool,
    pub frame_count: u64,
    pub blockchain_confirmations: HashMap<String, u64>,
    pub tamper_evidence: Option<String>,
    pub court_report: CourtReport,
}

#[derive(Debug)]
pub struct CourtReport {
    pub evidence_id: String,
    pub chain_of_custody: Vec<CustodyEntry>,
    pub cryptographic_proofs: Vec<String>,
    pub legal_compliance: LegalCompliance,
    pub generated_at: u64,
}

#[derive(Debug)]
pub struct CustodyEntry {
    pub timestamp: u64,
    pub actor: String,
    pub action: String,
    pub signature: String,
    pub blockchain_reference: String,
}

#[derive(Debug)]
pub struct LegalCompliance {
    pub standards_met: Vec<String>,
    pub certifications: Vec<String>,
    pub jurisdiction_compliance: Vec<String>,
}

pub type FrameSender = mpsc::UnboundedSender<VideoFrame>;
pub type FrameReceiver = mpsc::UnboundedReceiver<VideoFrame>;
pub type EncryptedFrameSender = mpsc::UnboundedSender<EncryptedFrame>;
pub type EncryptedFrameReceiver = mpsc::UnboundedReceiver<EncryptedFrame>;

#[async_trait::async_trait]
pub trait EncryptionEngine {
    async fn encrypt_frame(&mut self, frame: VideoFrame) -> Result<EncryptedFrame>;
    async fn decrypt_frame(&self, encrypted: &EncryptedFrame) -> Result<VideoFrame>;
    async fn verify_integrity(&self, frames: &[EncryptedFrame]) -> Result<VerificationResult>;
}

#[async_trait::async_trait]
pub trait BlockchainAnchor {
    async fn anchor_hash(&self, hash: &str, metadata: &FrameMetadata) -> Result<BlockchainAnchor>;
    async fn verify_anchor(&self, anchor: &BlockchainAnchor) -> Result<bool>;
    async fn get_confirmation_count(&self, tx_hash: &str) -> Result<u64>;
}

#[async_trait::async_trait]
pub trait StorageBackend {
    async fn store_frame(&self, frame: &EncryptedFrame) -> Result<String>;
    async fn retrieve_frame(&self, frame_id: &str) -> Result<EncryptedFrame>;
    async fn store_metadata(&self, metadata: &CourtReport) -> Result<String>;
}
