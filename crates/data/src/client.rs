use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RpcError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("RPC error response: {code} - {message}")]
    RpcErrorResponse { code: i64, message: String },
    #[error("Missing expected field in RPC response: {0}")]
    MissingField(&'static str),
}

#[derive(Debug, Clone)]
pub struct SolanaRpcClient {
    client: Client,
    url: String,
}

#[derive(Serialize)]
struct RpcRequest<'a> {
    jsonrpc: &'static str,
    id: u64,
    method: &'a str,
    params: Value,
}

impl SolanaRpcClient {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            url: url.into(),
        }
    }

    async fn send_request(&self, method: &str, params: Value) -> Result<Value, RpcError> {
        let request = RpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method,
            params,
        };

        let response: Value = self
            .client
            .post(&self.url)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        if let Some(err) = response.get("error") {
            return Err(RpcError::RpcErrorResponse {
                code: err.get("code").and_then(|c| c.as_i64()).unwrap_or(-1),
                message: err
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error")
                    .to_string(),
            });
        }

        response
            .get("result")
            .cloned()
            .ok_or(RpcError::MissingField("result"))
    }

    pub async fn get_slot(&self) -> Result<u64, RpcError> {
        let result = self.send_request("getSlot", json!([])).await?;
        result.as_u64().ok_or(RpcError::MissingField("slot (u64)"))
    }

    pub async fn get_recent_prioritization_fees(&self) -> Result<Vec<u64>, RpcError> {
        let result = self
            .send_request("getRecentPrioritizationFees", json!([]))
            .await?;

        let mut fees = Vec::new();
        if let Some(array) = result.as_array() {
            for item in array {
                if let Some(fee) = item.get("prioritizationFee").and_then(|f| f.as_u64()) {
                    fees.push(fee);
                }
            }
        }
        Ok(fees)
    }

    pub async fn get_latest_blockhash(&self) -> Result<(String, u64), RpcError> {
        let result = self
            .send_request(
                "getLatestBlockhash",
                json!([{"commitment": "finalized"}]),
            )
            .await?;

        let context = result.get("context").ok_or(RpcError::MissingField("context"))?;
        let slot = context.get("slot").and_then(|s| s.as_u64()).ok_or(RpcError::MissingField("context.slot"))?;

        let value = result.get("value").ok_or(RpcError::MissingField("value"))?;
        let blockhash = value
            .get("blockhash")
            .and_then(|b| b.as_str())
            .ok_or(RpcError::MissingField("value.blockhash"))?
            .to_string();

        Ok((blockhash, slot))
    }

    pub async fn get_blocks_with_limit(&self, start_slot: u64, limit: u64) -> Result<Vec<u64>, RpcError> {
        let result = self
            .send_request("getBlocksWithLimit", json!([start_slot, limit]))
            .await?;

        let mut blocks = Vec::new();
        if let Some(array) = result.as_array() {
            for item in array {
                if let Some(slot) = item.as_u64() {
                    blocks.push(slot);
                }
            }
        }
        Ok(blocks)
    }

    pub async fn get_block_time(&self, slot: u64) -> Result<Option<i64>, RpcError> {
        let result = match self.send_request("getBlockTime", json!([slot])).await {
            Ok(v) => v,
            Err(RpcError::RpcErrorResponse { .. }) => return Ok(None), // Slot might be skipped or not have a time
            Err(e) => return Err(e),
        };
        Ok(result.as_i64())
    }
}
