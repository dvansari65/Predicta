use network::{NetworkProfile, FeeMarketLevel, CongestionLevel, BlockhashWindowStatus, PacketDeliveryQuality};
use tx_model::{TransactionProfile, BlockhashFreshness};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimulationResult {
    /// 0.0 to 1.0 probability of landing
    pub landing_probability: f32,
    /// Estimated delay in slots before landing
    pub estimated_delay_slots: u16,
    /// Specific risks identified
    pub risk_reasons: Vec<RiskReason>,
    /// Assessment of the fee
    pub fee_adequacy: FeeAdequacy,
    /// Advice on retries
    pub retry_advice: RetryAdvice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskReason {
    FeeTooLow,
    BlockhashExpiring,
    HighCongestion,
    PacketLossHigh,
    LargeTransactionSize,
    HeavyAccountContention,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeeAdequacy {
    Underfunded,
    Competitive,
    Overfunded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetryAdvice {
    DoNotRetry,
    RetryImmediately,
    WaitAndSee,
}

pub struct Simulator;

impl Simulator {
    pub fn simulate(tx: &TransactionProfile, network: &NetworkProfile) -> SimulationResult {
        let mut probability: f32 = 1.0;
        let mut delay_slots: u16 = network.confirmation_delay_slots_p50;
        let mut risks = Vec::new();

        // 1. Fee Assessment
        let fee_adequacy = Self::assess_fee(tx.priority_fee_microlamports, network);
        match fee_adequacy {
            FeeAdequacy::Underfunded => {
                probability *= 0.3; // Huge penalty
                delay_slots = delay_slots.saturating_add(10);
                risks.push(RiskReason::FeeTooLow);
            }
            FeeAdequacy::Competitive => {
                // Baseline
            }
            FeeAdequacy::Overfunded => {
                probability = (probability + 0.1).min(1.0); // Slight boost
            }
        }

        // 2. Congestion Assessment
        match network.congestion_level {
            CongestionLevel::Low => {}
            CongestionLevel::Moderate => {
                probability *= 0.95;
                delay_slots = delay_slots.saturating_add(2);
            }
            CongestionLevel::High => {
                probability *= 0.7;
                delay_slots = delay_slots.saturating_add(5);
                risks.push(RiskReason::HighCongestion);
            }
            CongestionLevel::Severe => {
                probability *= 0.4;
                delay_slots = delay_slots.saturating_add(15);
                risks.push(RiskReason::HighCongestion);
            }
        }

        // 3. Blockhash Age
        if tx.blockhash_freshness == BlockhashFreshness::Stale || network.blockhash_window_status == BlockhashWindowStatus::Exhausted {
            probability = 0.0;
            risks.push(RiskReason::BlockhashExpiring);
        } else if tx.blockhash_freshness == BlockhashFreshness::Aging || network.blockhash_window_status == BlockhashWindowStatus::Tight {
            probability *= 0.6;
            risks.push(RiskReason::BlockhashExpiring);
        }

        // 4. Packet Delivery Quality
        if network.packet_delivery_quality == PacketDeliveryQuality::Degraded {
            probability *= 0.5;
            risks.push(RiskReason::PacketLossHigh);
        }

        // 5. Transaction Size Penalty (Minor)
        if tx.tx_size_bytes > 1000 {
            probability *= 0.95;
            risks.push(RiskReason::LargeTransactionSize);
        }

        // 6. Contention Penalty (Minor)
        if tx.writable_account_count > 5 || tx.compute_unit_limit > 500_000 {
            probability *= 0.90;
            risks.push(RiskReason::HeavyAccountContention);
        }

        // Floor probability at 0.0
        probability = probability.max(0.0).min(1.0);

        // Retry Advice Logic
        let retry_advice = if probability == 0.0 {
            RetryAdvice::DoNotRetry
        } else if probability < 0.5 {
            if network.congestion_level == CongestionLevel::Severe {
                RetryAdvice::WaitAndSee
            } else {
                RetryAdvice::RetryImmediately
            }
        } else {
            RetryAdvice::WaitAndSee
        };

        SimulationResult {
            landing_probability: probability,
            estimated_delay_slots: delay_slots,
            risk_reasons: risks,
            fee_adequacy,
            retry_advice,
        }
    }

    fn assess_fee(tx_fee: u64, network: &NetworkProfile) -> FeeAdequacy {
        if network.median_prioritization_fee_microlamports == 0 {
            return if tx_fee > 0 { FeeAdequacy::Overfunded } else { FeeAdequacy::Competitive };
        }

        let ratio = (tx_fee as f64) / (network.median_prioritization_fee_microlamports as f64);

        if ratio < 0.5 {
            FeeAdequacy::Underfunded
        } else if ratio > 2.0 {
            FeeAdequacy::Overfunded
        } else {
            FeeAdequacy::Competitive
        }
    }
}
