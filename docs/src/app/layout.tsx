import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Predicta | Solana TX Predictor",
  description: "A high-performance Rust toolkit for analyzing Solana transactions before submission.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>
        <nav style={{ padding: '1.5rem 2rem', display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid var(--border-color)', background: 'var(--bg-glass)', backdropFilter: 'blur(10px)', position: 'sticky', top: 0, zIndex: 100 }}>
          <div style={{ fontWeight: 800, fontSize: '1.2rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <span style={{ color: 'var(--accent-green)' }}>⬡</span> Predicta
          </div>
          <div style={{ display: 'flex', gap: '2rem', alignItems: 'center' }}>
            <a href="/">Home</a>
            <a href="/getting-started">Documentation</a>
            <a href="https://github.com/dvansari65/Predicta" target="_blank" className="btn btn-secondary" style={{ padding: '0.4rem 1rem' }}>GitHub</a>
          </div>
        </nav>
        <main>
          {children}
        </main>
      </body>
    </html>
  );
}
