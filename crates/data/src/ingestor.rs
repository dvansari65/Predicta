use crate::client::{SolanaRpcClient, RpcError};
use network::NetworkSnapshot;
use thiserror::Error;
use std::time::Duration;

pub const DEFAULT_BLOCKHASH_VALIDITY_WINDOW: u64 = 150;

#[derive(Error, Debug)]
pub enum IngestionError {
    #[error("RPC error during ingestion: {0}")]
    Rpc(#[from] RpcError),
    #[error("Not enough slot time samples available")]
    NotEnoughSlotSamples,
}

pub struct RpcIngestor {
    client: SolanaRpcClient,
}

impl RpcIngestor {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            client: SolanaRpcClient::new(url),
        }
    }

    /// Fetches live data from the configured RPC and constructs a NetworkSnapshot.
    /// Note: Some fields (dropped packet rate, confirmation delay, pending txs)
    /// are difficult to accurately measure via public RPCs and are currently set to 0.
    pub async fn fetch_snapshot(&self) -> Result<NetworkSnapshot, IngestionError> {
        let current_slot = self.client.get_slot().await?;
        
        let (_, latest_blockhash_context_slot) = self.client.get_latest_blockhash().await?;
        let recent_prioritization_fees_microlamports = self.client.get_recent_prioritization_fees().await?;

        let recent_slot_time_samples_ms = self.fetch_recent_slot_times(current_slot).await?;

        Ok(NetworkSnapshot {
            current_slot,
            recent_slot_time_samples_ms,
            recent_prioritization_fees_microlamports,
            blockhash_validity_window_slots: DEFAULT_BLOCKHASH_VALIDITY_WINDOW,
            latest_blockhash_context_slot,
            // These metrics typically require specialized validator plugins or deep analytics platforms.
            // For a basic predictor, we default to best-case values.
            pending_transaction_estimate: 0,
            dropped_packet_rate_bps: 0,
            confirmation_delay_slots_p50: 0,
            confirmation_delay_slots_p90: 0,
        })
    }

    async fn fetch_recent_slot_times(&self, current_slot: u64) -> Result<Vec<u32>, IngestionError> {
        // Fetch up to 10 recent blocks to get some time samples.
        // We look a little bit behind the current slot to ensure the blocks actually exist
        // and aren't skipped slots at the very tip.
        let start_slot = current_slot.saturating_sub(10);
        let recent_blocks = self.client.get_blocks_with_limit(start_slot, 10).await?;

        if recent_blocks.len() < 2 {
            return Err(IngestionError::NotEnoughSlotSamples);
        }

        let mut block_times = Vec::new();
        for slot in &recent_blocks {
            if let Some(time) = self.client.get_block_time(*slot).await? {
                block_times.push((*slot, time));
            }
        }

        // We need at least 2 consecutive times to calculate a diff
        if block_times.len() < 2 {
            return Err(IngestionError::NotEnoughSlotSamples);
        }

        // Sort by slot just in case
        block_times.sort_by_key(|(s, _)| *s);

        let mut samples_ms = Vec::new();
        for i in 1..block_times.len() {
            let (prev_slot, prev_time) = block_times[i - 1];
            let (curr_slot, curr_time) = block_times[i];

            let slot_diff = curr_slot - prev_slot;
            if slot_diff == 0 {
                continue;
            }

            let time_diff_secs = curr_time - prev_time;
            
            // Calculate average time per slot in this diff window
            // and convert to milliseconds
            if time_diff_secs >= 0 {
                let ms_per_slot = (time_diff_secs as u64 * 1000) / slot_diff;
                // Avoid storing 0 ms slot times, as NetworkSnapshot validation rejects them.
                if ms_per_slot > 0 {
                    samples_ms.push(ms_per_slot as u32);
                }
            }
        }

        if samples_ms.is_empty() {
            return Err(IngestionError::NotEnoughSlotSamples);
        }

        Ok(samples_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    use serde_json::json;

    #[tokio::test]
    async fn test_fetch_snapshot_success() {
        let mut server = Server::new_async().await;
        let url = server.url();

        // We mock a single catch-all POST endpoint and return different JSON-RPC responses based on the method
        // This is a bit tricky with mockito without advanced matchers, so we will create specific mocks
        // based on the body matching the method name.

        let _mock_slot = server.mock("POST", "/")
            .match_body(mockito::Matcher::Regex("getSlot".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({"jsonrpc": "2.0", "result": 100, "id": 1}).to_string())
            .create_async().await;

        let _mock_blockhash = server.mock("POST", "/")
            .match_body(mockito::Matcher::Regex("getLatestBlockhash".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({
                "jsonrpc": "2.0",
                "result": {
                    "context": { "slot": 98 },
                    "value": { "blockhash": "dummyhash", "lastValidBlockHeight": 123 }
                },
                "id": 1
            }).to_string())
            .create_async().await;

        let _mock_fees = server.mock("POST", "/")
            .match_body(mockito::Matcher::Regex("getRecentPrioritizationFees".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({
                "jsonrpc": "2.0",
                "result": [
                    {"prioritizationFee": 100, "slot": 90},
                    {"prioritizationFee": 200, "slot": 91}
                ],
                "id": 1
            }).to_string())
            .create_async().await;

        let _mock_blocks = server.mock("POST", "/")
            .match_body(mockito::Matcher::Regex("getBlocksWithLimit".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({
                "jsonrpc": "2.0",
                "result": [90, 91, 92],
                "id": 1
            }).to_string())
            .create_async().await;

        let _mock_time1 = server.mock("POST", "/")
            .match_body(mockito::Matcher::Regex("getBlockTime.*90".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({"jsonrpc": "2.0", "result": 1600000000, "id": 1}).to_string())
            .create_async().await;

        let _mock_time2 = server.mock("POST", "/")
            .match_body(mockito::Matcher::Regex("getBlockTime.*91".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({"jsonrpc": "2.0", "result": 1600000001, "id": 1}).to_string())
            .create_async().await;
            
        let _mock_time3 = server.mock("POST", "/")
            .match_body(mockito::Matcher::Regex("getBlockTime.*92".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({"jsonrpc": "2.0", "result": 1600000001, "id": 1}).to_string()) // diff 0s for test coverage
            .create_async().await;

        let ingestor = RpcIngestor::new(&url);
        let snapshot = ingestor.fetch_snapshot().await.expect("Failed to fetch snapshot");

        assert_eq!(snapshot.current_slot, 100);
        assert_eq!(snapshot.latest_blockhash_context_slot, 98);
        assert_eq!(snapshot.recent_prioritization_fees_microlamports, vec![100, 200]);
        // Slot diff between 90 and 91 is 1. Time diff is 1 second (1000ms).
        assert_eq!(snapshot.recent_slot_time_samples_ms, vec![1000]);
    }
}
