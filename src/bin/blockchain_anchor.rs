use clap::{Arg, Command};
use std::collections::HashMap;
use std::fs;
use tracing::{error, info, warn};
use tracing_subscriber;

use immutable_encryption::{
    blockchain::{BlockchainConfig, MultiChainAnchor},
    config::Config,
    FrameMetadata,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Parse command line arguments
    let matches = Command::new("blockchain-anchor")
        .version("0.1.0")
        .about("Standalone blockchain anchoring tool")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file path"),
        )
        .arg(
            Arg::new("hash")
                .short('h')
                .long("hash")
                .value_name("HASH")
                .help("Hash to anchor to blockchain")
                .required(true),
        )
        .arg(
            Arg::new("metadata")
                .short('m')
                .long("metadata")
                .value_name("FILE")
                .help("JSON file containing frame metadata"),
        )
        .arg(
            Arg::new("device-id")
                .long("device-id")
                .value_name("ID")
                .help("Device ID")
                .default_value("standalone_anchor"),
        )
        .arg(
            Arg::new("chains")
                .long("chains")
                .value_name("CHAINS")
                .help("Comma-separated list of chains (ethereum,bitcoin,private)")
                .default_value("ethereum,bitcoin"),
        )
        .arg(
            Arg::new("verify")
                .short('v')
                .long("verify")
                .value_name("ANCHOR_FILE")
                .help("Verify existing anchor from JSON file"),
        )
        .get_matches();

    // Load configuration
    let config = if let Some(config_path) = matches.get_one::<String>("config") {
        Config::load_from_file(config_path)?
    } else {
        Config::load()?
    };

    // Validate configuration
    config.validate()?;

    // Initialize blockchain anchor
    let blockchain_config = config.get_blockchain_config();
    let anchor = MultiChainAnchor::new(blockchain_config).await?;

    if let Some(anchor_file) = matches.get_one::<String>("verify") {
        // Verify mode
        verify_anchor(&anchor, anchor_file).await?;
    } else {
        // Anchor mode
        let hash = matches.get_one::<String>("hash").unwrap();

        // Load or create metadata
        let metadata = if let Some(metadata_file) = matches.get_one::<String>("metadata") {
            load_metadata_from_file(metadata_file)?
        } else {
            let device_id = matches.get_one::<String>("device-id").unwrap();
            create_default_metadata(device_id)
        };

        anchor_hash(&anchor, hash, &metadata).await?;
    }

    Ok(())
}

async fn anchor_hash(
    anchor: &MultiChainAnchor,
    hash: &str,
    metadata: &FrameMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Anchoring hash to blockchain: {}", hash);

    let result = anchor.anchor_to_all_chains(hash, metadata).await?;

    println!("Anchoring Results:");
    for anchor_result in &result {
        println!("Chain: {}", anchor_result.chain);
        println!("Transaction Hash: {}", anchor_result.transaction_hash);
        println!("Block Number: {}", anchor_result.block_number);
        println!("Timestamp: {}", anchor_result.timestamp);
        println!("Proof: {}", anchor_result.proof);
        println!("---");
    }

    // Save results to file
    let output_file = format!("anchor_{}.json", hash);
    let json = serde_json::to_string_pretty(&result)?;
    fs::write(&output_file, &json)?;
    info!("Anchoring results saved to: {}", output_file);

    Ok(())
}

async fn verify_anchor(
    anchor: &MultiChainAnchor,
    anchor_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Verifying anchor from file: {}", anchor_file);

    let content = fs::read_to_string(anchor_file)?;
    let anchors: Vec<immutable_encryption::BlockchainAnchor> = serde_json::from_str(&content)?;

    let results = anchor.verify_all_anchors(&anchors).await?;

    println!("Verification Results:");
    for anchor_data in &anchors {
        println!("Chain: {}", anchor_data.chain);

        if let Some(is_valid) = results.get(&anchor_data.chain) {
            if *is_valid {
                println!("✓ Verification successful");

                // Get confirmation count - need to use the specific anchor
                let confirmations = match anchor_data.chain.as_str() {
                    "bitcoin" => anchor
                        .bitcoin
                        .get_confirmation_count(&anchor_data.transaction_hash)
                        .await
                        .unwrap_or(0),
                    "ethereum" => anchor
                        .ethereum
                        .get_confirmation_count(&anchor_data.transaction_hash)
                        .await
                        .unwrap_or(0),
                    _ => 0,
                };
                println!("Confirmations: {}", confirmations);
            } else {
                println!("✗ Verification failed");
            }
        } else {
            println!("? Verification result unknown");
        }
        println!("---");
    }

    Ok(())
}

fn load_metadata_from_file(file_path: &str) -> Result<FrameMetadata, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let metadata: FrameMetadata = serde_json::from_str(&content)?;
    Ok(metadata)
}

fn create_default_metadata(device_id: &str) -> FrameMetadata {
    FrameMetadata {
        device_id: device_id.to_string(),
        location: None,
        resolution: (1920, 1080),
        fps: 30,
        codec: "H.264".to_string(),
    }
}
