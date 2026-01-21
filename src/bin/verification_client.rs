use clap::{Arg, Command};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Parse command line arguments
    let matches = Command::new("verification-client")
        .version("0.1.0")
        .about("Client for verifying immutable encrypted evidence")
        .arg(
            Arg::new("server")
                .short('s')
                .long("server")
                .value_name("URL")
                .help("Server URL (default: http://localhost:8080)")
                .default_value("http://localhost:8080"),
        )
        .arg(
            Arg::new("evidence")
                .short('e')
                .long("evidence")
                .value_name("ID")
                .help("Evidence ID to verify")
                .required(true),
        )
        .arg(
            Arg::new("court-report")
                .short('c')
                .long("court-report")
                .help("Generate court report instead of basic verification"),
        )
        .arg(
            Arg::new("watch")
                .short('w')
                .long("watch")
                .help("Watch for verification updates"),
        )
        .get_matches();

    let server_url = matches.get_one::<String>("server").unwrap();
    let evidence_id = matches.get_one::<String>("evidence").unwrap();
    let generate_court_report = matches.get_flag("court-report");
    let watch_mode = matches.get_flag("watch");

    info!("Connecting to verification server at {}", server_url);

    let client = Client::new();

    if watch_mode {
        watch_verification(&client, server_url, evidence_id).await?;
    } else if generate_court_report {
        generate_court_report_request(&client, server_url, evidence_id).await?;
    } else {
        verify_evidence(&client, server_url, evidence_id).await?;
    }

    Ok(())
}

async fn verify_evidence(
    client: &Client,
    server_url: &str,
    evidence_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Verifying evidence: {}", evidence_id);

    let url = format!("{}/verify/{}", server_url, evidence_id);

    let response = client.get(&url).send().await?;

    if response.status().is_success() {
        let result: Value = response.json().await?;
        println!("Verification Result:");
        println!("{}", serde_json::to_string_pretty(&result)?);

        if let Some(is_valid) = result.get("is_valid") {
            if is_valid.as_bool().unwrap_or(false) {
                info!("✓ Evidence verification successful");
            } else {
                warn!("✗ Evidence verification failed");
            }
        }
    } else {
        error!("Verification request failed: {}", response.status());
        let error_text = response.text().await?;
        println!("Error: {}", error_text);
    }

    Ok(())
}

async fn generate_court_report_request(
    client: &Client,
    server_url: &str,
    evidence_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Generating court report for evidence: {}", evidence_id);

    let url = format!("{}/court-report/{}", server_url, evidence_id);

    let response = client.get(&url).send().await?;

    if response.status().is_success() {
        let result: Value = response.json().await?;
        println!("Court Report:");
        println!("{}", serde_json::to_string_pretty(&result)?);
        info!("✓ Court report generated successfully");
    } else {
        error!("Court report request failed: {}", response.status());
        let error_text = response.text().await?;
        println!("Error: {}", error_text);
    }

    Ok(())
}

async fn watch_verification(
    client: &Client,
    server_url: &str,
    evidence_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Watching verification status for evidence: {}", evidence_id);

    loop {
        let url = format!("{}/verify/{}", server_url, evidence_id);

        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let result: Value = response.json().await?;
                    println!("Status Update:");
                    println!("{}", serde_json::to_string_pretty(&result)?);

                    if let Some(is_valid) = result.get("is_valid") {
                        if is_valid.as_bool().unwrap_or(false) {
                            info!("✓ Verification completed successfully");
                            break;
                        }
                    }
                } else {
                    println!("Status: Verification in progress...");
                }
            }
            Err(e) => {
                error!("Failed to check verification status: {}", e);
            }
        }

        sleep(Duration::from_secs(5)).await;
    }

    Ok(())
}
