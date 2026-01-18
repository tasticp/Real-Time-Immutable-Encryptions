use clap::{Arg, Command};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};
use tracing_subscriber;

use immutable_encryption::{config::Config, FrameMetadata, RealTimeEncryptionNode, VideoFrame};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Parse command line arguments
    let matches = Command::new("encryption-node")
        .version("0.1.0")
        .about("Real-time immutable video encryption and blockchain anchoring")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file path"),
        )
        .arg(
            Arg::new("demo")
                .short('d')
                .long("demo")
                .help("Run in demo mode with simulated video frames"),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("Server port"),
        )
        .get_matches();

    // Load configuration
    let config = if let Some(config_path) = matches.get_one::<String>("config") {
        Config::load_from_file(config_path)?
    } else {
        Config::load()?
    };

    // Override port if provided
    let mut config = config;
    if let Some(port) = matches.get_one::<String>("port") {
        config.server.port = port.parse().map_err(|e| format!("Invalid port: {}", e))?;
    }

    info!(
        "Starting Immutable Encryption Node on port {}",
        config.server.port
    );

    // Validate configuration
    config.validate()?;

    // Initialize the encryption node
    let node = RealTimeEncryptionNode::new(
        config.get_crypto_config(),
        config.get_blockchain_config(),
        config.get_storage_config(),
        config.get_verification_config(),
    )
    .await?;

    // Start the processing pipeline
    let (frame_sender, encrypted_receiver) = node.start_processing().await?;

    // Start demo mode if requested
    if matches.get_flag("demo") {
        info!("Starting demo mode with simulated video frames");
        tokio::spawn(async move {
            demo_video_generation(frame_sender).await;
        });
    }

    // Start HTTP server for API endpoints
    start_http_server(config, node).await?;

    Ok(())
}

async fn demo_video_generation(sender: immutable_encryption::FrameSender) {
    let mut sequence = 0;
    let mut interval = tokio::time::interval(Duration::from_millis(33)); // ~30 FPS

    loop {
        interval.tick().await;

        sequence += 1;

        // Simulate video frame data
        let frame_data = vec![0u8; 1024 * 1024]; // 1MB frame
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let frame = VideoFrame {
            timestamp,
            sequence,
            data: frame_data,
            metadata: FrameMetadata {
                device_id: "demo_drone_001".to_string(),
                location: Some((40.7128 + (sequence as f64 * 0.0001), -74.0060)), // Moving coordinates
                resolution: (1920, 1080),
                fps: 30,
                codec: "H.264".to_string(),
            },
        };

        if let Err(e) = sender.send(frame) {
            error!("Failed to send demo frame: {}", e);
            break;
        }

        if sequence % 100 == 0 {
            info!("Generated {} demo frames", sequence);
        }

        // Stop after 10 minutes for demo
        if sequence >= 18000 {
            info!("Demo completed after {} frames", sequence);
            break;
        }
    }
}

async fn start_http_server(
    config: Config,
    node: RealTimeEncryptionNode,
) -> Result<(), Box<dyn std::error::Error>> {
    use warp::Filter;

    info!(
        "Starting HTTP server on {}:{}",
        config.server.host, config.server.port
    );

    // Health check endpoint
    let health = warp::path("health").and(warp::get()).map(|| {
        warp::reply::json(&serde_json::json!({
            "status": "healthy",
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        }))
    });

    // Status endpoint
    let node_clone = node.clone();
    let status = warp::path("status").and(warp::get()).map(move || {
        warp::reply::json(&serde_json::json!({
            "node": "running",
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        }))
    });

    // Verify evidence endpoint
    let node_clone = node.clone();
    let verify = warp::path("verify")
        .and(warp::path::param::<String>())
        .and(warp::get())
        .and_then(move |evidence_id: String| {
            let node = node_clone.clone();
            async move {
                match node.verify_evidence(&[evidence_id]).await {
                    Ok(result) => Ok(warp::reply::json(&result)),
                    Err(e) => {
                        error!("Verification failed: {}", e);
                        Ok(warp::reply::json(&serde_json::json!({
                            "error": e.to_string()
                        })))
                    }
                }
            }
        });

    // Generate court report endpoint
    let node_clone = node.clone();
    let court_report = warp::path("court-report")
        .and(warp::path::param::<String>())
        .and(warp::get())
        .and_then(move |evidence_id: String| {
            let node = node_clone.clone();
            async move {
                match node.generate_court_report(&evidence_id).await {
                    Ok(report) => Ok(warp::reply::json(&report)),
                    Err(e) => {
                        error!("Court report generation failed: {}", e);
                        Ok(warp::reply::json(&serde_json::json!({
                            "error": e.to_string()
                        })))
                    }
                }
            }
        });

    // Combine all routes
    let routes = health
        .or(status)
        .or(verify)
        .or(court_report)
        .with(warp::cors().allow_any_origin())
        .with(warp::log("api"));

    // Start server
    warp::serve(routes)
        .run((
            config.server.host.parse::<std::net::IpAddr>()?,
            config.server.port,
        ))
        .await;

    Ok(())
}
