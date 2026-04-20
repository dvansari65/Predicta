<div align="center">
  <h1><code>Predicta</code></h1>
  <h2>🚀 Solana TX Predictor</h2>

  <p>
    A high-performance Rust toolkit for analyzing Solana transactions <strong>before submission</strong>,
    providing probabilistic insights into execution outcomes under real network conditions.
  </p>
</div>

---

## ✨ Overview

Solana transaction execution is highly dependent on dynamic runtime conditions such as:

* Network congestion
* Leader scheduling
* Priority fee competition
* Transaction timing

This project provides a **pre-submission analysis engine** that helps developers and bots make informed decisions before sending transactions.

---

## 🏗️ Project Structure

This project is built as a modular workspace, allowing developers to use only the pieces they need:
- `tx-model`: Domain model for validating and profiling transactions.
- `network`: Domain model for defining network state and congestion levels.
- `data`: Live data ingestion from Solana RPCs to populate the network state.
- `simulator`: The core engine that scores a transaction against the network state.
- `cli`: A command-line interface for testing predictions without writing code.

---

## 💻 Developer Usage & Examples

If you are building a trading bot, wallet backend, or DeFi application, you can use the `simulator` crate to intelligently adjust priority fees and retry strategies dynamically.

### 1. Add to your `Cargo.toml`
If you are integrating this into your own project, depend on the crates you need:

```toml
[dependencies]
tx-model = { path = "crates/tx-model" }
data = { path = "crates/data" }
network = { path = "crates/network" }
simulator = { path = "crates/simulator" }
tokio = { version = "1.52", features = ["full"] }
```

### 2. End-to-End Prediction Example (e.g., Trading Bot)

This example demonstrates exactly how a Rust backend or Trading Bot uses the library to predict landing probability and assess fees *before* broadcasting a transaction.

```rust
use data::RpcIngestor;
use simulator::{Simulator, FeeAdequacy, RetryAdvice, RiskReason};
use tx_model::{AccountMeta, Instruction, Transaction};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Build your transaction (Domain Model representation)
    let my_tx = Transaction {
        instructions: vec![Instruction {
            program_id: "JUP6LkbZbjS1jKKwapdH67yIeI2tWcbkXJ5A2e8YpXF".to_string(), // Jupiter Swap
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
        priority_fee_microlamports: 5_000, // We are guessing 5,000 is enough
        tx_size_bytes: 450,
        recent_blockhash_age_slots: 2, // Very fresh blockhash
    };

    // 2. Validate and Profile the Transaction
    let tx_profile = my_tx.profile().expect("Invalid transaction structure");

    // 3. Fetch Live Network Conditions (e.g., from Mainnet)
    println!("Fetching live network state...");
    let ingestor = RpcIngestor::new("https://api.mainnet-beta.solana.com");
    let network_snapshot = ingestor.fetch_snapshot().await.expect("RPC fetch failed");
    let network_profile = network_snapshot.profile().expect("Invalid network state");

    // 4. Run the Simulator
    let result = Simulator::simulate(&tx_profile, &network_profile);

    // 5. Make programmatic decisions based on the result
    println!("Landing Probability: {:.1}%", result.landing_probability * 100.0);
    println!("Estimated Delay: {} slots", result.estimated_delay_slots);
    
    match result.fee_adequacy {
        FeeAdequacy::Underfunded => {
            println!("Warning: Your fee is too low for current network conditions!");
            // Bot logic: Automatically bump the priority fee and re-simulate
        }
        FeeAdequacy::Competitive => println!("Fee is competitive."),
        FeeAdequacy::Overfunded => println!("Fee is generous, high chance of landing quickly."),
    }

    if result.risk_reasons.contains(&RiskReason::HighCongestion) {
        println!("Warning: The network is currently highly congested.");
        // Bot logic: Alert the user that the swap might take longer than usual
    }

    match result.retry_advice {
        RetryAdvice::DoNotRetry => println!("Action: Drop this transaction. Blockhash is dead."),
        RetryAdvice::WaitAndSee => println!("Action: Send it, but wait a few slots before retrying."),
        RetryAdvice::RetryImmediately => println!("Action: Send and aggressively spam retries."),
    }

    Ok(())
}
```

---

## 🚀 Usage (CLI)

If you don't want to write Rust code and just want to test the math, you can use the built-in CLI tool.

### 1. Generate a sample transaction JSON
```bash
cargo run -p cli -- sample > my_tx.json
```

### 2. Run the prediction engine
This will read your JSON file, fetch real-time data from mainnet, and print a color-coded analysis to your terminal.
```bash
cargo run -p cli -- predict --tx-file my_tx.json
```

---

## 🧪 Development

### Run tests
```bash
cargo test --workspace
```

### Format code
```bash
cargo fmt
```

### Lint
```bash
cargo clippy
```

---

## ⚠️ Disclaimer

This project provides probabilistic estimates and does not guarantee transaction outcomes on the Solana network.

---

## 🤝 Contributing

Contributions are welcome. Please open issues or submit pull requests.

---

## 📄 License

MIT License
