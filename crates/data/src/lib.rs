pub mod client;
pub mod ingestor;

pub use client::{RpcError, SolanaRpcClient};
pub use ingestor::{IngestionError, RpcIngestor};

#[cfg(test)]
mod tests {
    // Note: Since we are using reqwest to make actual HTTP calls,
    // true unit testing of the RpcIngestor would require standing up a mock HTTP server
    // (e.g. using `mockito` or `wiremock` crates).
    // For now, we will add a simple placeholder test, and rely on integration tests
    // or manual verification against a live testnet/mainnet node.

    #[test]
    fn test_ingestor_initialization() {
        let _ingestor = super::RpcIngestor::new("http://localhost:8899");
        // Successfully initialized
    }
}
