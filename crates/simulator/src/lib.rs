pub mod engine;

pub use engine::{Simulator, SimulationResult, RiskReason, FeeAdequacy, RetryAdvice};

#[cfg(test)]
mod tests {
    use super::*;
    use network::{NetworkProfile, FeeMarketLevel, CongestionLevel, BlockhashWindowStatus, PacketDeliveryQuality, SlotHealth};
    use tx_model::{TransactionProfile, BlockhashFreshness};

    fn default_tx_profile() -> TransactionProfile {
        TransactionProfile {
            instruction_count: 1,
            signer_count: 1,
            writable_account_count: 1,
            readonly_account_count: 1,
            writable_signer_count: 1,
            readonly_signer_count: 0,
            nonsigner_writable_account_count: 0,
            nonsigner_readonly_account_count: 1,
            compute_unit_limit: 200_000,
            priority_fee_microlamports: 5_000,
            tx_size_bytes: 400,
            recent_blockhash_age_slots: 10,
            blockhash_freshness: BlockhashFreshness::Fresh,
            unique_program_ids: vec![],
            writable_accounts: vec![],
            readonly_accounts: vec![],
            signer_accounts: vec![],
        }
    }

    fn default_network_profile() -> NetworkProfile {
        NetworkProfile {
            current_slot: 1000,
            average_slot_time_ms: 400,
            slot_time_jitter_ms: 50,
            median_prioritization_fee_microlamports: 5_000,
            min_prioritization_fee_microlamports: 1_000,
            max_prioritization_fee_microlamports: 10_000,
            blockhash_age_slots: 10,
            blockhash_headroom_slots: 140,
            blockhash_window_status: BlockhashWindowStatus::Healthy,
            pending_transaction_estimate: 1000,
            dropped_packet_rate_bps: 0,
            packet_delivery_quality: PacketDeliveryQuality::Healthy,
            confirmation_delay_slots_p50: 1,
            confirmation_delay_slots_p90: 2,
            slot_health: SlotHealth::Stable,
            fee_market_level: FeeMarketLevel::Competitive,
            congestion_level: CongestionLevel::Low,
        }
    }

    #[test]
    fn test_perfect_conditions() {
        let tx = default_tx_profile();
        let net = default_network_profile();
        
        let result = Simulator::simulate(&tx, &net);
        
        assert_eq!(result.fee_adequacy, FeeAdequacy::Competitive);
        assert_eq!(result.landing_probability, 1.0);
        assert!(result.risk_reasons.is_empty());
        assert_eq!(result.estimated_delay_slots, 1);
        assert_eq!(result.retry_advice, RetryAdvice::WaitAndSee);
    }

    #[test]
    fn test_underfunded_fee() {
        let mut tx = default_tx_profile();
        tx.priority_fee_microlamports = 1000;
        let net = default_network_profile();

        let result = Simulator::simulate(&tx, &net);
        
        assert_eq!(result.fee_adequacy, FeeAdequacy::Underfunded);
        assert!(result.landing_probability < 0.5);
        assert!(result.risk_reasons.contains(&RiskReason::FeeTooLow));
        assert!(result.estimated_delay_slots > 5);
        assert_eq!(result.retry_advice, RetryAdvice::RetryImmediately);
    }

    #[test]
    fn test_stale_blockhash() {
        let mut tx = default_tx_profile();
        tx.blockhash_freshness = BlockhashFreshness::Stale;
        let net = default_network_profile();

        let result = Simulator::simulate(&tx, &net);
        
        assert_eq!(result.landing_probability, 0.0);
        assert!(result.risk_reasons.contains(&RiskReason::BlockhashExpiring));
        assert_eq!(result.retry_advice, RetryAdvice::DoNotRetry);
    }

    #[test]
    fn test_severe_congestion() {
        let tx = default_tx_profile();
        let mut net = default_network_profile();
        net.congestion_level = CongestionLevel::Severe;

        let result = Simulator::simulate(&tx, &net);
        
        assert!(result.landing_probability < 0.5);
        assert!(result.risk_reasons.contains(&RiskReason::HighCongestion));
        assert_eq!(result.retry_advice, RetryAdvice::WaitAndSee); // Should wait and see during severe congestion
    }

    #[test]
    fn test_heavy_contention() {
        let mut tx = default_tx_profile();
        tx.compute_unit_limit = 1_000_000;
        tx.writable_account_count = 10;
        let net = default_network_profile();

        let result = Simulator::simulate(&tx, &net);
        
        assert!(result.landing_probability < 1.0);
        assert!(result.risk_reasons.contains(&RiskReason::HeavyAccountContention));
    }
}
