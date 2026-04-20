import Link from 'next/link';

export default function Home() {
  return (
    <div style={{ minHeight: '90vh', display: 'flex', flexDirection: 'column', alignItems: 'center', padding: '4rem 2rem' }}>
      
      {/* Hero Section */}
      <section style={{ textAlign: 'center', maxWidth: '800px', marginTop: '4rem', marginBottom: '6rem' }} className="animate-fade-in">
        <div style={{ display: 'inline-block', padding: '0.4rem 1rem', borderRadius: '50px', background: 'rgba(20, 241, 149, 0.1)', border: '1px solid rgba(20, 241, 149, 0.3)', color: 'var(--accent-green)', fontSize: '0.9rem', marginBottom: '2rem', fontWeight: 600 }}>
          🚀 Version 0.1.0 is live on crates.io
        </div>
        <h1 style={{ fontSize: '4rem', lineHeight: 1.1, marginBottom: '1.5rem' }}>
          Stop Guessing on <br /><span className="gradient-text">Solana Transactions</span>
        </h1>
        <p style={{ fontSize: '1.25rem', color: 'var(--text-secondary)', marginBottom: '3rem', maxWidth: '600px', margin: '0 auto 3rem auto' }}>
          A high-performance Rust library that predicts landing probabilities, estimates delays, and recommends priority fees <i>before</i> you broadcast to the network.
        </p>
        
        <div style={{ display: 'flex', gap: '1rem', justifyContent: 'center' }}>
          <Link href="/getting-started" className="btn btn-primary">
            Read the Docs
          </Link>
          <a href="https://crates.io/crates/predicta-simulator" target="_blank" rel="noreferrer" className="btn btn-secondary">
            View on Crates.io
          </a>
        </div>
      </section>

      {/* Features Section */}
      <section style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))', gap: '2rem', maxWidth: '1000px', width: '100%' }} className="animate-fade-in delay-1">
        
        <div className="glass-panel">
          <div style={{ fontSize: '2rem', marginBottom: '1rem' }}>🔮</div>
          <h3>Probabilistic Landing</h3>
          <p style={{ color: 'var(--text-secondary)' }}>
            Know exactly what the chances are of your transaction surviving network congestion, based on live RPC data.
          </p>
        </div>

        <div className="glass-panel">
          <div style={{ fontSize: '2rem', marginBottom: '1rem' }}>💰</div>
          <h3>Fee Adequacy Engine</h3>
          <p style={{ color: 'var(--text-secondary)' }}>
            Stop overpaying or underpaying. Predicta compares your priority fee against the current network median in real-time.
          </p>
        </div>

        <div className="glass-panel">
          <div style={{ fontSize: '2rem', marginBottom: '1rem' }}>🛡️</div>
          <h3>Risk Flagging</h3>
          <p style={{ color: 'var(--text-secondary)' }}>
            Automatically detect stale blockhashes, severe packet loss environments, and heavy account contention before signing.
          </p>
        </div>

      </section>
    </div>
  );
}
