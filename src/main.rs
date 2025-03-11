use eyre::Result;
use axal::{chain_data::ChainComparisonConfig, prover::{Prover, STANDARD_CONFIG}};
use clap::Parser;
use tokio::fs::File;
use tokio::io::AsyncWriteExt; 
use std::path::PathBuf;

// Define command line arguments using clap
#[derive(Parser)]
#[clap(author, version, about = "Plonky2 chain comparison tool")]
struct Cli {
    /// Path to the JSON configuration file
    #[clap(short, long)]
    config: PathBuf,

    /// Optional output path for the proof
    #[clap(short, long)]
    output: Option<PathBuf>,

    /// Optional flag for verbose output
    #[clap(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {

    // Parse command line arguments
    let cli = Cli::parse();
    
    let config_data = tokio::fs::read_to_string(&cli.config)
        .await?;

    let chains: ChainComparisonConfig = serde_json::from_str(&config_data)?;

    let mut prover = Prover::new(STANDARD_CONFIG);
    let proof = prover.prove(chains).await?;

    // Serialize to a string first
    let proof_json = serde_json::to_string_pretty(&proof)?;

    // Then use async write
    let mut file = File::create("proof.json").await?;
    file.write_all(proof_json.as_bytes()).await?;
    
    Ok(())
}
