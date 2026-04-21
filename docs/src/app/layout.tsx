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
        <nav className="nav-bar">
          <div className="nav-logo">
            <img src="/logo.svg" alt="Predicta Logo" width="32" height="32" />
            Predicta
          </div>
          <div className="nav-links">
            <a href="/">Home</a>
            <a href="/getting-started">Documentation</a>
            <a href="https://github.com/dvansari65/Predicta" target="_blank" className="btn btn-secondary nav-btn">GitHub</a>
          </div>
        </nav>
        <main>
          {children}
        </main>
      </body>
    </html>
  );
}
