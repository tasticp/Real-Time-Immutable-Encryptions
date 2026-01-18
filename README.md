# Real-Time Immutable Encryption System

A comprehensive real-time video encryption and verification system with blockchain anchoring for court-admissible evidence.

# Quick start with Docker Compose
cd scripts
chmod +x deploy.sh
./deploy.sh latest docker-compose

# Access services
# Rust Backend: http://localhost:8080
# Python API: http://localhost:8000/docs
# Streamlit UI: http://localhost:8501
# Grafana: http://localhost:3000

## Architecture

```
[Drones/Cameras] → [Edge Processing] → [Multi-Chain Anchoring] → [Verification Portal]
```

## Features

- **Real-time video encryption** with frame-level cryptographic proofs
- **Multi-chain blockchain anchoring** (Bitcoin, Ethereum, Private Chain)
- **Zero-knowledge verification** for privacy-preserving proof
- **Quantum-resistant cryptography** for future-proofing
- **Court-admissible evidence** chain of custody
- **Hardware security integration** (TPM, HSM)

## Quick Start

### Rust Core
```bash
cargo build --release
cargo run --bin encryption-node
```

### Python API
```bash
pip install -r requirements.txt
python -m python_api.main
```

### Web Interface
```bash
cd python_api/streamlit
streamlit run app.py
```

## Security Features

- SHA-256 + BLAKE3 double hashing
- AES-256-GCM encryption
- Post-quantum Kyber key exchange
- Multi-factor authentication
- Hardware security module support

## Blockchain Integration

- **Bitcoin**: Primary timestamp anchoring
- **Ethereum**: Smart contract verification
- **Hyperledger Fabric**: Private organization chain
- **OpenTimestamps**: Decentralized backup

## Evidence Verification

1. Frame-level hash chain verification
2. Blockchain timestamp validation
3. Zero-knowledge proof verification
4. Hardware attestation check
5. Court-ready report generation

## Performance

- **Latency**: <100ms encryption + <5s blockchain anchoring
- **Throughput**: 4K@30fps per device
- **Storage overhead**: <10% for metadata
- **Verification time**: <1s per segment

## License

MIT License - see LICENSE file for details
