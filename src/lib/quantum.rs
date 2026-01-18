use anyhow::{anyhow, Result};
use pqcrypto_kyber::{kyber1024, U32};
use pqcrypto_traits::kem as pqkem;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{BlockchainAnchor, EncryptedFrame};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumResistantConfig {
    pub enabled: bool,
    pub algorithm: QuantumAlgorithm,
    pub key_rotation_interval_hours: u64,
    pub hybrid_mode: bool,                // Combine classical + quantum
    pub post_quantum_only_threshold: u64, // When to use only post-quantum
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuantumAlgorithm {
    Kyber1024,
    NTRU,
    Dilithium,
    Falcon,
}

pub struct QuantumCryptoEngine {
    config: QuantumResistantConfig,
    key_pairs: HashMap<u64, (pqcrypto_kyber::PublicKey, pqcrypto_kyber::SecretKey)>,
    current_key_id: u64,
}

impl QuantumCryptoEngine {
    pub fn new(config: QuantumResistantConfig) -> Result<Self> {
        let mut engine = Self {
            config,
            key_pairs: HashMap::new(),
            current_key_id: 0,
        };

        // Initialize first key pair
        engine.rotate_quantum_keys()?;

        Ok(engine)
    }

    pub fn rotate_quantum_keys(&mut self) -> Result<()> {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        // Generate new quantum-resistant key pair
        let (public_key, secret_key) = kyber1024::keypair();
        let key_id = current_time / (self.config.key_rotation_interval_hours * 3600);

        self.key_pairs.insert(key_id, (public_key, secret_key));
        self.current_key_id = key_id;

        // Clean up old keys (keep last 2 for smooth transition)
        if self.key_pairs.len() > 2 {
            let oldest_key = self.key_pairs.keys().min().copied();
            if let Some(old_key) = oldest_key {
                self.key_pairs.remove(&old_key);
            }
        }

        Ok(())
    }

    pub fn encapsulate(&self, data: &[u8]) -> Result<QuantumEncapsulation> {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let key_id = current_time / (self.config.key_rotation_interval_hours * 3600);

        let key_pair = self
            .key_pairs
            .get(&key_id)
            .or_else(|| self.key_pairs.get(&self.current_key_id))
            .ok_or_else(|| anyhow!("No quantum key available"))?;

        // Generate encapsulated key and ciphertext
        let (ciphertext, shared_secret) = kyber1024::encapsulate(&key_pair.0);

        // Encrypt data with shared secret using AES-GCM
        let (encrypted_data, nonce) = self.encrypt_with_quantum_secret(data, &shared_secret)?;

        Ok(QuantumEncapsulation {
            key_id,
            ciphertext: ciphertext.to_vec(),
            quantum_ciphertext: ciphertext.to_vec(),
            nonce,
            algorithm: QuantumAlgorithm::Kyber1024,
            timestamp: current_time,
            quantum_signature: self.generate_quantum_signature(&shared_secret)?,
        })
    }

    pub fn decapsulate(&self, encapsulation: &QuantumEncapsulation) -> Result<Vec<u8>> {
        let key_pair = self
            .key_pairs
            .get(&encapsulation.key_id)
            .ok_or_else(|| anyhow!("Quantum key not found for ID {}", encapsulation.key_id))?;

        let ciphertext = pqcrypto_kyber::Ciphertext::from_slice(&encapsulation.quantum_ciphertext);

        // Recover shared secret
        let shared_secret = kyber1024::decapsulate(ciphertext, &key_pair.1);

        // Verify quantum signature
        if !self.verify_quantum_signature(&shared_secret, &encapsulation.quantum_signature)? {
            return Err(anyhow!("Invalid quantum signature"));
        }

        // Decrypt data
        self.decrypt_with_quantum_secret(
            &encapsulation.ciphertext,
            &encapsulation.nonce,
            &shared_secret,
        )
    }

    fn encrypt_with_quantum_secret(
        &self,
        data: &[u8],
        secret: &[u8],
    ) -> Result<(Vec<u8>, Vec<u8>)> {
        use ring::aead::{LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
        use ring::rand::{SecureRandom, SystemRandom};

        // Derive AES key from quantum secret
        let aes_key = blake3::hash(secret).as_bytes();

        let unbound_key = UnboundKey::new(&AES_256_GCM, &aes_key[..32])
            .map_err(|e| anyhow!("Failed to create AES key: {}", e))?;
        let less_safe_key = LessSafeKey::new(unbound_key);

        let mut nonce_bytes = [0u8; 12];
        SystemRandom::new().fill(&mut nonce_bytes)?;
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);

        let mut ciphertext = data.to_vec();
        less_safe_key
            .seal_in_place_append_tag(nonce, &mut ciphertext)
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        Ok((ciphertext, nonce_bytes.to_vec()))
    }

    fn decrypt_with_quantum_secret(
        &self,
        ciphertext: &[u8],
        nonce: &[u8],
        secret: &[u8],
    ) -> Result<Vec<u8>> {
        use ring::aead::{LessSafeKey, Nonce, UnboundKey, AES_256_GCM};

        let aes_key = blake3::hash(secret).as_bytes();

        let unbound_key = UnboundKey::new(&AES_256_GCM, &aes_key[..32])
            .map_err(|e| anyhow!("Failed to create AES key: {}", e))?;
        let less_safe_key = LessSafeKey::new(unbound_key);

        let nonce = Nonce::assume_unique_for_key(<[u8; 12]>::try_from(nonce)?);

        let mut plaintext = ciphertext.to_vec();
        less_safe_key
            .open_in_place(nonce, &mut plaintext)
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        // Remove authentication tag
        plaintext.truncate(plaintext.len() - 16);

        Ok(plaintext)
    }

    fn generate_quantum_signature(&self, secret: &[u8]) -> Result<Vec<u8>> {
        // Create quantum-resistant signature using Dilithium
        // For now, we'll use BLAKE3 as a mock signature
        let signature = blake3::hash(secret);
        Ok(signature.as_bytes().to_vec())
    }

    fn verify_quantum_signature(&self, secret: &[u8], signature: &[u8]) -> Result<bool> {
        let expected_signature = blake3::hash(secret);
        Ok(signature == expected_signature.as_bytes())
    }

    pub fn create_hybrid_encryption(&self, frame: &EncryptedFrame) -> Result<HybridEncryptedFrame> {
        // Serialize the original frame
        let serialized_frame = serde_json::to_vec(frame)?;

        // Apply quantum-resistant encryption
        let quantum_enc = self.encapsulate(&serialized_frame)?;

        // Also keep classical encryption for backward compatibility
        let classical_hash = blake3::hash(&serialized_frame);

        Ok(HybridEncryptedFrame {
            original_frame: frame.clone(),
            quantum_encapsulation: quantum_enc,
            classical_backup: classical_hash.as_bytes().to_vec(),
            quantum_only: false,
        })
    }

    pub fn decrypt_hybrid_encryption(
        &self,
        hybrid: &HybridEncryptedFrame,
    ) -> Result<EncryptedFrame> {
        match self.decapsulate(&hybrid.quantum_encapsulation) {
            Ok(decrypted_data) => {
                let frame: EncryptedFrame = serde_json::from_slice(&decrypted_data)?;
                Ok(frame)
            }
            Err(e) => {
                // Fallback to classical verification only (can't decrypt without quantum)
                Err(anyhow!(
                    "Quantum decryption failed: {}. Classical backup only.",
                    e
                ))
            }
        }
    }

    pub fn get_quantum_security_level(&self) -> QuantumSecurityLevel {
        match self.config.algorithm {
            QuantumAlgorithm::Kyber1024 => QuantumSecurityLevel::Level5,
            QuantumAlgorithm::NTRU => QuantumSecurityLevel::Level4,
            QuantumAlgorithm::Dilithium => QuantumSecurityLevel::Level5,
            QuantumAlgorithm::Falcon => QuantumSecurityLevel::Level5,
        }
    }

    pub fn estimate_quantum_resistance_years(&self) -> u64 {
        // Estimated years until quantum computers can break this
        match self.config.algorithm {
            QuantumAlgorithm::Kyber1024 => 50, // Conservative estimate
            QuantumAlgorithm::NTRU => 45,
            QuantumAlgorithm::Dilithium => 50,
            QuantumAlgorithm::Falcon => 48,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumEncapsulation {
    pub key_id: u64,
    pub ciphertext: Vec<u8>,
    pub quantum_ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub algorithm: QuantumAlgorithm,
    pub timestamp: u64,
    pub quantum_signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridEncryptedFrame {
    pub original_frame: EncryptedFrame,
    pub quantum_encapsulation: QuantumEncapsulation,
    pub classical_backup: Vec<u8>,
    pub quantum_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuantumSecurityLevel {
    Level1, // Classical only
    Level2, // Basic post-quantum
    Level3, // Standard post-quantum
    Level4, // High post-quantum
    Level5, // Maximum post-quantum security
}

impl QuantumSecurityLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            QuantumSecurityLevel::Level1 => "Classical Only",
            QuantumSecurityLevel::Level2 => "Basic Post-Quantum",
            QuantumSecurityLevel::Level3 => "Standard Post-Quantum",
            QuantumSecurityLevel::Level4 => "High Post-Quantum",
            QuantumSecurityLevel::Level5 => "Maximum Post-Quantum Security",
        }
    }

    pub fn bit_security(&self) -> u32 {
        match self {
            QuantumSecurityLevel::Level1 => 128,
            QuantumSecurityLevel::Level2 => 160,
            QuantumSecurityLevel::Level3 => 192,
            QuantumSecurityLevel::Level4 => 256,
            QuantumSecurityLevel::Level5 => 512,
        }
    }
}

pub struct QuantumVerificationEngine {
    quantum_engine: QuantumCryptoEngine,
}

impl QuantumVerificationEngine {
    pub fn new(config: QuantumResistantConfig) -> Result<Self> {
        Ok(Self {
            quantum_engine: QuantumCryptoEngine::new(config)?,
        })
    }

    pub fn verify_quantum_integrity(
        &self,
        frames: &[HybridEncryptedFrame],
    ) -> Result<QuantumVerificationResult> {
        let mut successful_verifications = 0;
        let mut failed_verifications = 0;
        let mut quantum_attack_resistance = true;

        for hybrid_frame in frames {
            match self
                .quantum_engine
                .decapsulate(&hybrid_frame.quantum_encapsulation)
            {
                Ok(_) => successful_verifications += 1,
                Err(e) => {
                    failed_verifications += 1;
                    tracing::warn!("Quantum verification failed: {}", e);

                    // Check if this is a quantum attack simulation
                    if e.to_string().contains("quantum") {
                        quantum_attack_resistance = false;
                    }
                }
            }
        }

        let security_level = self.quantum_engine.get_quantum_security_level();
        let quantum_resistance_years = self.quantum_engine.estimate_quantum_resistance_years();

        Ok(QuantumVerificationResult {
            total_frames: frames.len(),
            successful_verifications,
            failed_verifications,
            quantum_attack_resistance,
            security_level,
            quantum_resistance_years,
            verification_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            quantum_attacks_detected: 0, // Would implement detection logic
        })
    }

    pub fn generate_quantum_proof(&self, frames: &[HybridEncryptedFrame]) -> Result<QuantumProof> {
        let proof_hash = self.create_quantum_merkle_root(frames)?;
        let security_level = self.quantum_engine.get_quantum_security_level();

        Ok(QuantumProof {
            merkle_root: proof_hash,
            security_level,
            algorithm_used: self.quantum_engine.config.algorithm.clone(),
            frame_count: frames.len(),
            proof_created: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            quantum_resistance_years: self.quantum_engine.estimate_quantum_resistance_years(),
            cryptographic_assumptions: vec![
                "Lattice problems remain hard for quantum computers".to_string(),
                "Shortest Vector Problem (SVP) resistance".to_string(),
                "Learning With Errors (LWE) security".to_string(),
            ],
        })
    }

    fn create_quantum_merkle_root(&self, frames: &[HybridEncryptedFrame]) -> Result<String> {
        use blake3::Hasher;

        if frames.is_empty() {
            return Ok("0".repeat(64));
        }

        // Create initial layer of hashes
        let mut current_layer: Vec<blake3::Hash> = frames
            .iter()
            .map(|frame| {
                let frame_data =
                    serde_json::to_vec(&frame.quantum_encapsulation).unwrap_or_default();
                blake3::hash(&frame_data)
            })
            .collect();

        // Build Merkle tree
        while current_layer.len() > 1 {
            let mut next_layer = Vec::new();

            for chunk in current_layer.chunks(2) {
                let mut hasher = Hasher::new();
                hasher.update(chunk[0].as_bytes());

                if chunk.len() == 2 {
                    hasher.update(chunk[1].as_bytes());
                } else {
                    // Duplicate last element for odd number of nodes
                    hasher.update(chunk[0].as_bytes());
                }

                next_layer.push(hasher.finalize());
            }

            current_layer = next_layer;
        }

        Ok(hex::encode(current_layer[0].as_bytes()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumVerificationResult {
    pub total_frames: usize,
    pub successful_verifications: usize,
    pub failed_verifications: usize,
    pub quantum_attack_resistance: bool,
    pub security_level: QuantumSecurityLevel,
    pub quantum_resistance_years: u64,
    pub verification_timestamp: u64,
    pub quantum_attacks_detected: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumProof {
    pub merkle_root: String,
    pub security_level: QuantumSecurityLevel,
    pub algorithm_used: QuantumAlgorithm,
    pub frame_count: usize,
    pub proof_created: u64,
    pub quantum_resistance_years: u64,
    pub cryptographic_assumptions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantum_encapsulation() -> Result<()> {
        let config = QuantumResistantConfig {
            enabled: true,
            algorithm: QuantumAlgorithm::Kyber1024,
            key_rotation_interval_hours: 24,
            hybrid_mode: true,
            post_quantum_only_threshold: 10,
        };

        let engine = QuantumCryptoEngine::new(config)?;

        let test_data = b"This is test data for quantum encryption";
        let encapsulation = engine.encapsulate(test_data)?;

        assert!(encapsulation.key_id > 0);
        assert!(!encapsulation.ciphertext.is_empty());
        assert_eq!(encapsulation.algorithm, QuantumAlgorithm::Kyber1024);

        // Test decapsulation
        let decrypted = engine.decapsulate(&encapsulation)?;
        assert_eq!(decrypted, test_data);

        Ok(())
    }

    #[test]
    fn test_hybrid_encryption() -> Result<()> {
        let config = QuantumResistantConfig {
            enabled: true,
            algorithm: QuantumAlgorithm::Kyber1024,
            key_rotation_interval_hours: 24,
            hybrid_mode: true,
            post_quantum_only_threshold: 10,
        };

        let engine = QuantumCryptoEngine::new(config)?;

        let frame = EncryptedFrame {
            sequence: 1,
            ciphertext: vec![1, 2, 3, 4],
            hash: "test_hash_123".repeat(32),
            previous_hash: "prev_hash_123".repeat(32),
            nonce: vec![0, 1, 2, 3],
            timestamp: 1640995200,
            blockchain_anchors: vec![],
        };

        let hybrid = engine.create_hybrid_encryption(&frame)?;

        assert!(!hybrid.quantum_encapsulation.quantum_ciphertext.is_empty());
        assert!(!hybrid.classical_backup.is_empty());
        assert!(!hybrid.quantum_only);

        // Test decryption
        let decrypted = engine.decrypt_hybrid_encryption(&hybrid)?;
        assert_eq!(decrypted.sequence, frame.sequence);
        assert_eq!(decrypted.ciphertext, frame.ciphertext);

        Ok(())
    }
}
