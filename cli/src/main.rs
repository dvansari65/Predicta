use clap::{Parser, Subcommand};
use colored::*;
use data::RpcIngestor;
use simulator::Simulator;
use std::fs;
use std::path::PathBuf;
use tx_model::{AccountMeta, Instruction, Transaction};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Predict landing success for a transaction
    Predict {
        /// Path to the JSON file containing the transaction details
        #[arg(short, long)]
        tx_file: PathBuf,

        /// RPC URL to use for network data
        #[arg(long, default_value = "https://api.mainnet-beta.solana.com")]
        rpc_url: String,
    },
    /// Generate a sample transaction JSON file
    Sample,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Predict { tx_file, rpc_url } => {
            println!("{} reading transaction from {:?}", "➔".blue(), tx_file);
            let tx_json = fs::read_to_string(tx_file)?;
            let tx: Transaction = serde_json::from_str(&tx_json)?;

            println!("{} validating transaction...", "➔".blue());
            let tx_profile = match tx.profile() {
                Ok(p) => p,
                Err(e) => {
                    println!("{} Transaction validation failed: {}", "✗".red(), e);
                    return Ok(());
                }
            };

            println!("{} fetching network state from {}...", "➔".blue(), rpc_url);
            let ingestor = RpcIngestor::new(rpc_url);
            let snapshot = match ingestor.fetch_snapshot().await {
                Ok(s) => s,
                Err(e) => {
                    println!("{} Failed to fetch network state: {}", "✗".red(), e);
                    return Ok(());
                }
            };

            let net_profile = match snapshot.profile() {
                Ok(p) => p,
                Err(e) => {
                    println!("{} Network state validation failed: {}", "✗".red(), e);
                    return Ok(());
                }
            };

            println!("{} running simulation...", "➔".blue());
            let result = Simulator::simulate(&tx_profile, &net_profile);

            println!("\n{}", "=== Simulation Results ===".bold());
            
            let prob = result.landing_probability * 100.0;
            let prob_str = format!("{:.1}%", prob);
            let prob_colored = if prob > 90.0 {
                prob_str.green()
            } else if prob > 50.0 {
                prob_str.yellow()
            } else {
                prob_str.red()
            };
            println!("{:<25} {}", "Landing Probability:".bold(), prob_colored);
            
            println!("{:<25} {} slots", "Estimated Delay:".bold(), result.estimated_delay_slots.to_string().yellow());
            
            let fee_str = format!("{:?}", result.fee_adequacy);
            let fee_colored = match result.fee_adequacy {
                simulator::FeeAdequacy::Competitive | simulator::FeeAdequacy::Overfunded => fee_str.green(),
                simulator::FeeAdequacy::Underfunded => fee_str.red(),
            };
            println!("{:<25} {}", "Fee Adequacy:".bold(), fee_colored);
            
            let advice_str = format!("{:?}", result.retry_advice);
            println!("{:<25} {}", "Retry Advice:".bold(), advice_str.cyan());

            if !result.risk_reasons.is_empty() {
                println!("\n{}", "Identified Risks:".red().bold());
                for risk in result.risk_reasons {
                    println!("  {} {:?}", "-".red(), risk);
                }
            } else {
                println!("\n{} No significant risks identified.", "✓".green());
            }
        }
        Commands::Sample => {
            let sample_tx = Transaction {
                instructions: vec![Instruction {
                    program_id: "ComputeBudget111111111111111111111111111111".to_string(),
                    accounts: vec!["payer".to_string()],
                }],
                accounts: vec![AccountMeta {
                    pubkey: "payer".to_string(),
                    is_signer: true,
                    is_writable: true,
                }],
                compute_unit_limit: 200_000,
                priority_fee_microlamports: 10_000,
                tx_size_bytes: 400,
                recent_blockhash_age_slots: 5,
            };

            let json = serde_json::to_string_pretty(&sample_tx)?;
            println!("{}", json);
        }
    }

    Ok(())
}
