"use client";

import Link from 'next/link';
import { useEffect, useState } from 'react';

export default function GettingStarted() {
  const [activeSection, setActiveSection] = useState('getting-started');

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            setActiveSection(entry.target.id);
          }
        });
      },
      { rootMargin: '-20% 0px -80% 0px' } // Triggers when section is near top of viewport
    );

    const sections = document.querySelectorAll('section[id]');
    sections.forEach((section) => observer.observe(section));

    // Also observe the top of the page to reset to "getting-started"
    const topElement = document.getElementById('top-of-page');
    if (topElement) observer.observe(topElement);

    return () => {
      sections.forEach((section) => observer.unobserve(section));
      if (topElement) observer.unobserve(topElement);
    };
  }, []);

  return (
    <div style={{ display: 'flex', minHeight: '100vh', maxWidth: '1200px', margin: '0 auto' }}>
      
      {/* Sidebar */}
      <aside style={{ width: '250px', padding: '2rem 1rem', borderRight: '1px solid var(--border-color)', position: 'sticky', top: '80px', height: 'calc(100vh - 80px)' }}>
        <h3 style={{ fontSize: '0.9rem', textTransform: 'uppercase', color: 'var(--text-secondary)', letterSpacing: '0.05em', marginBottom: '1rem' }}>Documentation</h3>
        <ul style={{ listStyle: 'none', display: 'flex', flexDirection: 'column', gap: '0.8rem' }}>
          <li>
            <Link href="#top-of-page" style={{ 
              color: activeSection === 'top-of-page' || activeSection === 'getting-started' ? 'var(--accent-green)' : 'var(--text-secondary)', 
              fontWeight: activeSection === 'top-of-page' || activeSection === 'getting-started' ? 600 : 400,
              transition: 'all 0.2s ease'
            }}>Getting Started</Link>
          </li>
          <li>
            <Link href="#installation" style={{ 
              color: activeSection === 'installation' ? 'var(--accent-green)' : 'var(--text-secondary)', 
              fontWeight: activeSection === 'installation' ? 600 : 400,
              transition: 'all 0.2s ease'
            }}>Installation</Link>
          </li>
          <li>
            <Link href="#cli-usage" style={{ 
              color: activeSection === 'cli-usage' ? 'var(--accent-green)' : 'var(--text-secondary)', 
              fontWeight: activeSection === 'cli-usage' ? 600 : 400,
              transition: 'all 0.2s ease'
            }}>CLI Usage</Link>
          </li>
          <li>
            <Link href="#rust-usage" style={{ 
              color: activeSection === 'rust-usage' ? 'var(--accent-green)' : 'var(--text-secondary)', 
              fontWeight: activeSection === 'rust-usage' ? 600 : 400,
              transition: 'all 0.2s ease'
            }}>Rust Library Usage</Link>
          </li>
        </ul>
      </aside>

      {/* Main Content */}
      <main style={{ flex: 1, padding: '3rem 4rem', maxWidth: '800px' }} className="animate-fade-in">
        <div id="top-of-page">
          <h1 style={{ fontSize: '3rem', marginBottom: '1rem' }}>Getting Started</h1>
          <p style={{ fontSize: '1.2rem', color: 'var(--text-secondary)', marginBottom: '3rem' }}>
            Learn how to install Predicta and integrate it into your Solana trading bots, wallets, or DeFi backends.
          </p>
        </div>

        <section id="installation" style={{ marginBottom: '4rem', paddingTop: '2rem' }}>
          <h2>1. Installation</h2>
          <p style={{ marginBottom: '1rem' }}>
            Predicta is published on <a href="https://crates.io/crates/predicta" target="_blank" style={{ color: 'var(--accent-purple)', textDecoration: 'underline' }}>crates.io</a>. It is a fully bundled suite, so you just need to run:
          </p>
          <pre><code>{`cargo add predicta`}</code></pre>
          <p style={{ marginTop: '1rem', color: 'var(--text-secondary)', fontSize: '0.9rem' }}>
            This will download the entire suite (simulator, data ingestion, tx-model, and network context).
          </p>
        </section>

        <section id="cli-usage" style={{ marginBottom: '4rem', paddingTop: '2rem' }}>
          <h2>2. CLI Usage (Testing)</h2>
          <p style={{ marginBottom: '1rem' }}>
            If you want to quickly test the engine without writing Rust code, you can use the built-in CLI tool to simulate a JSON transaction against live Mainnet data.
          </p>
          
          <div className="glass-panel" style={{ marginBottom: '1.5rem', padding: '1.5rem' }}>
            <h4 style={{ marginBottom: '0.5rem', color: 'var(--accent-green)' }}>Step 1: Generate a sample</h4>
            <pre style={{ background: 'rgba(0,0,0,0.3) !important', border: 'none' }}><code>{`cargo run -p predicta-cli -- sample > my_tx.json`}</code></pre>
          </div>

          <div className="glass-panel" style={{ padding: '1.5rem' }}>
            <h4 style={{ marginBottom: '0.5rem', color: 'var(--accent-green)' }}>Step 2: Run the prediction</h4>
            <pre style={{ background: 'rgba(0,0,0,0.3) !important', border: 'none' }}><code>{`cargo run -p predicta-cli -- predict --tx-file my_tx.json`}</code></pre>
          </div>
        </section>

        <section id="rust-usage" style={{ marginBottom: '4rem', paddingTop: '2rem' }}>
          <h2>3. Rust Integration Example</h2>
          <p style={{ marginBottom: '1rem' }}>
            This is how a Trading Bot or Wallet Backend uses the library to predict landing probability and assess fees <i>before</i> broadcasting.
          </p>
          <pre><code>{`use predicta::data::RpcIngestor;
use predicta::simulator::{Simulator, FeeAdequacy, RetryAdvice, RiskReason};
use predicta::tx_model::{AccountMeta, Instruction, Transaction};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Build your transaction domain model
    let my_tx = Transaction {
        instructions: vec![Instruction {
            program_id: "JUP6LkbZbjS1jKKwapdH67yIeI2tWcbkXJ5A2e8YpXF".to_string(), // Jupiter
            accounts: vec!["payer".to_string(), "pool_address".to_string()],
        }],
        accounts: vec![
            AccountMeta { pubkey: "payer".to_string(), is_signer: true, is_writable: true },
            AccountMeta { pubkey: "pool_address".to_string(), is_signer: false, is_writable: true },
        ],
        compute_unit_limit: 300_000,
        priority_fee_microlamports: 5_000,
        tx_size_bytes: 450,
        recent_blockhash_age_slots: 2, 
    };

    // 2. Profile the Tx
    let tx_profile = my_tx.profile().unwrap();

    // 3. Fetch Live Network Conditions from Mainnet
    let ingestor = RpcIngestor::new("https://api.mainnet-beta.solana.com");
    let network_snapshot = ingestor.fetch_snapshot().await.unwrap();
    let network_profile = network_snapshot.profile().unwrap();

    // 4. Run the Simulator
    let result = Simulator::simulate(&tx_profile, &network_profile);

    println!("Landing Probability: {:.1}%", result.landing_probability * 100.0);
    println!("Estimated Delay: {} slots", result.estimated_delay_slots);
    
    match result.retry_advice {
        RetryAdvice::WaitAndSee => println!("Action: Send it, wait a few slots."),
        RetryAdvice::RetryImmediately => println!("Action: Send and spam retries."),
        _ => {}
    }

    Ok(())
}`}</code></pre>
        </section>

      </main>
    </div>
  );
}
