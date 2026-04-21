"use client";

import Link from 'next/link';
import { useRef } from 'react';
import gsap from 'gsap';
import { useGSAP } from '@gsap/react';
import { ScrollTrigger } from 'gsap/ScrollTrigger';

gsap.registerPlugin(ScrollTrigger);

export default function Home() {
  const container = useRef<HTMLDivElement>(null);

  useGSAP(() => {
    const tl = gsap.timeline({ defaults: { ease: 'power3.out' } });

    tl.from('.hero-badge', { y: 20, opacity: 0, duration: 0.8 })
      .from('.hero-title', { y: 30, opacity: 0, duration: 1 }, '-=0.6')
      .from('.hero-subtitle', { y: 20, opacity: 0, duration: 0.8 }, '-=0.6')
      .from('.hero-buttons', { y: 20, opacity: 0, duration: 0.8 }, '-=0.6')
      .from('.features-grid .glass-panel', { y: 40, opacity: 0, duration: 1, stagger: 0.15 }, '-=0.4');

    const scrollSections = gsap.utils.toArray('.scroll-section');

    scrollSections.forEach((section: any) => {
      gsap.from(section.querySelectorAll('.scroll-el'), {
        scrollTrigger: {
          trigger: section,
          start: 'top 80%',
          toggleActions: 'play none none reverse'
        },
        y: 50,
        opacity: 0,
        duration: 1,
        stagger: 0.2,
        ease: 'power3.out'
      });
    });
  }, { scope: container });

  return (
    <div
      ref={container}
      className="page-container"
      style={{
        padding: '0 1.2rem',
        maxWidth: '1200px',
        margin: '0 auto'
      }}
    >

      {/* HERO SECTION */}
      <section className="hero-section" style={{ textAlign: 'center', padding: '4rem 0 3rem' }}>

        <div
          className="hero-badge"
          style={{
            display: 'inline-block',
            padding: '0.4rem 1rem',
            borderRadius: '50px',
            border: '1px solid rgba(53, 92, 125, 0.2)',
            color: 'var(--text-secondary)',
            fontSize: '0.7rem',
            marginBottom: '1.5rem',
            fontWeight: 500,
            letterSpacing: '0.05em',
            textTransform: 'uppercase'
          }}
        >
          Version 0.1.0 is live on crates.io
        </div>

        <h1
          className="hero-title"
          style={{
            color: 'var(--text-primary)',
            fontSize: 'clamp(1.8rem, 5vw, 3.2rem)',
            lineHeight: 1.2,
            marginBottom: '1rem'
          }}
        >
          Stop Guessing on <br />
          <span style={{ color: 'var(--accent-blue)' }}>
            Solana Transactions
          </span>
        </h1>

        <p
          className="hero-subtitle"
          style={{
            fontSize: 'clamp(0.95rem, 2.5vw, 1.1rem)',
            color: 'var(--text-secondary)',
            marginBottom: '2rem',
            maxWidth: '500px',
            marginLeft: 'auto',
            marginRight: 'auto',
            fontWeight: 300,
            lineHeight: 1.8
          }}
        >
          A high-performance Rust library that predicts landing probabilities, estimates delays, and recommends priority fees before you broadcast to the network.
        </p>

        <div
          className="hero-buttons"
          style={{
            display: 'flex',
            flexWrap: 'wrap',
            gap: '0.8rem',
            justifyContent: 'center'
          }}
        >
          <Link href="/getting-started" className="btn btn-primary">
            Read the Docs
          </Link>
          <a href="https://crates.io/crates/predicta-simulator" target="_blank" rel="noreferrer" className="btn btn-secondary">
            View on Crates.io
          </a>
        </div>
      </section>

      {/* FEATURES GRID */}
      <section
        className="features-grid"
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))',
          gap: '1rem',
          marginTop: '3rem'
        }}
      >
        {[{
          title: 'Probabilistic Landing',
          desc: 'Know exactly what the chances are of your transaction surviving network congestion, based on live RPC data.'
        }, {
          title: 'Fee Adequacy Engine',
          desc: 'Stop overpaying or underpaying. Predicta compares your priority fee against the current network median in real-time.'
        }, {
          title: 'Risk Flagging',
          desc: 'Automatically detect stale blockhashes, severe packet loss environments, and heavy account contention before signing.'
        }].map((item, i) => (
          <div key={i} className="glass-panel" style={{ padding: '1.2rem' }}>
            <h3 style={{ fontSize: '1rem', marginBottom: '1rem' }}>{item.title}</h3>
            <p style={{ fontSize: '0.9rem', color: 'var(--text-secondary)' }}>{item.desc}</p>
          </div>
        ))}
      </section>

      {/* HOW IT WORKS */}
      <section
        className="scroll-section how-it-works-section"
        style={{
          display: 'grid',
          gap: '2rem',
          marginTop: '4rem'
        }}
      >
        <div className="scroll-el">
          <h2 style={{ fontSize: 'clamp(1.5rem, 4vw, 2.2rem)', marginBottom: '1rem' }}>
            Pre-Flight Checks for Your Bots
          </h2>
          <p style={{ color: 'var(--text-secondary)', fontSize: '0.95rem', marginBottom: '1rem', lineHeight: 1.8 }}>
            Whether you are building a high-frequency MEV searcher or a reliable backend paymaster, Predicta acts as your pre-flight safety mechanism.
          </p>
          <ul style={{ color: 'var(--text-secondary)', paddingLeft: '1.2rem', lineHeight: 1.8, fontSize: '0.9rem' }}>
            <li>Fetch live congestion states from Mainnet</li>
            <li>Profile your transaction logic locally</li>
            <li>Receive exact landing probabilities</li>
            <li>Dynamically adjust fees to guarantee inclusion</li>
          </ul>
        </div>

        <div className="scroll-el glass-panel" style={{ padding: '1rem' }}>
          <pre style={{ margin: 0 }}>
            <code style={{ fontSize: '0.75rem', lineHeight: 1.6, whiteSpace: 'pre-wrap' }}>
{`// 1. Profile your transaction
let tx_profile = my_tx.profile()?;

// 2. Fetch live network state
let network = ingestor.fetch_snapshot().await?;
let net_profile = network.profile()?;

// 3. Simulate locally before sending
let result = Simulator::simulate(
    &tx_profile, 
    &net_profile
);

if result.landing_probability > 0.8 {
    broadcast(my_tx);
} else {
    bump_priority_fee();
}`}
            </code>
          </pre>
        </div>
      </section>

      {/* CTA SECTION */}
      <section
        className="scroll-section cta-section"
        style={{ textAlign: 'center', padding: '3rem 0' }}
      >
        <h2
          className="scroll-el"
          style={{ fontSize: 'clamp(1.8rem, 5vw, 2.5rem)', marginBottom: '1rem' }}
        >
          Ready to optimize your RPC costs?
        </h2>

        <p
          className="scroll-el"
          style={{ fontSize: '0.95rem', color: 'var(--text-secondary)', marginBottom: '2rem' }}
        >
          Stop wasting money on dropped transactions. Integrate Predicta into your Rust backend today.
        </p>

        <div className="scroll-el">
          <Link
            href="/getting-started"
            className="btn btn-primary"
            style={{ padding: '0.7rem 1.8rem', fontSize: '0.95rem' }}
          >
            Start Building Now
          </Link>
        </div>
      </section>

    </div>
  );
}
