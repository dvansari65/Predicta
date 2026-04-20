use data::RpcIngestor;
use simulator::{FeeAdequacy, RetryAdvice, RiskReason, Simulator};
use tx_model::{AccountMeta, Instruction, Transaction};

/// To run this example from the workspace root:
/// cargo run -p simulator --example bot_usage
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting End-to-End Prediction Example...\n");

    // 1. Build your transaction (Domain Model representation)
    // In a real bot, you'd parse this from your actual solana_sdk::transaction::Transaction
    let my_tx = Transaction {
        instructions: vec![Instruction {
            program_id: "JUP6LkbZbjS1jKKwapdH67yIeI2tWcbkXJ5A2e8YpXF".to_string(), // Jupiter
            accounts: vec!["payer".to_string(), "pool_address".to_string()],
        }],
        accounts: vec![
            AccountMeta {
                pubkey: "payer".to_string(),
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: "pool_address".to_string(),
                is_signer: false,
                is_writable: true,
            },
        ],
        compute_unit_limit: 300_000,
        priority_fee_microlamports: 5_000,
        tx_size_bytes: 450,
        recent_blockhash_age_slots: 2, // Very fresh blockhash
    };

    // 2. Validate and Profile the Transaction
    let tx_profile = my_tx.profile().expect("Invalid transaction structure");

    // 3. Fetch Live Network Conditions from Mainnet
    println!("Fetching live network state from mainnet-beta...");
    let ingestor = RpcIngestor::new("https://api.mainnet-beta.solana.com");
    let network_snapshot = ingestor.fetch_snapshot().await.expect("RPC fetch failed");
    let network_profile = network_snapshot.profile().expect("Invalid network state");

    // 4. Run the Simulator
    println!("Simulating transaction against current network conditions...\n");
    let result = Simulator::simulate(&tx_profile, &network_profile);

    // 5. Make programmatic decisions based on the result
    println!("--- PREDICTION RESULTS ---");
    println!("Landing Probability: {:.1}%", result.landing_probability * 100.0);
    println!("Estimated Delay: {} slots", result.estimated_delay_slots);
    
    match result.fee_adequacy {
        FeeAdequacy::Underfunded => {
            println!("Fee Assessment: UNDERFUNDED. You should bump your priority fee.");
        }
        FeeAdequacy::Competitive => println!("Fee Assessment: COMPETITIVE."),
        FeeAdequacy::Overfunded => println!("Fee Assessment: OVERFUNDED. You are paying a premium."),
    }

    if result.risk_reasons.contains(&RiskReason::HighCongestion) {
        println!("Network Warning: The network is currently highly congested.");
    }

    match result.retry_advice {
        RetryAdvice::DoNotRetry => println!("Strategy: Drop this transaction. Blockhash is dead."),
        RetryAdvice::WaitAndSee => println!("Strategy: Send it, but wait a few slots before retrying."),
        RetryAdvice::RetryImmediately => println!("Strategy: Send and aggressively spam retries."),
    }

    Ok(())
}
