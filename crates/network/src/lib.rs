use serde::{Deserialize, Serialize};
use std::fmt;

pub const MAX_DROPPED_PACKET_RATE_BPS: u16 = 10_000;
pub const TIGHT_BLOCKHASH_HEADROOM_SLOTS: u64 = 20;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkSnapshot {
    pub current_slot: u64,
    pub recent_slot_time_samples_ms: Vec<u32>,
    pub recent_prioritization_fees_microlamports: Vec<u64>,
    pub blockhash_validity_window_slots: u64,
    pub latest_blockhash_context_slot: u64,
    pub pending_transaction_estimate: u32,
    pub dropped_packet_rate_bps: u16,
    pub confirmation_delay_slots_p50: u16,
    pub confirmation_delay_slots_p90: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkProfile {
    pub current_slot: u64,
    pub average_slot_time_ms: u32,
    pub slot_time_jitter_ms: u32,
    pub median_prioritization_fee_microlamports: u64,
    pub min_prioritization_fee_microlamports: u64,
    pub max_prioritization_fee_microlamports: u64,
    pub blockhash_age_slots: u64,
    pub blockhash_headroom_slots: u64,
    pub blockhash_window_status: BlockhashWindowStatus,
    pub pending_transaction_estimate: u32,
    pub dropped_packet_rate_bps: u16,
    pub packet_delivery_quality: PacketDeliveryQuality,
    pub confirmation_delay_slots_p50: u16,
    pub confirmation_delay_slots_p90: u16,
    pub slot_health: SlotHealth,
    pub fee_market_level: FeeMarketLevel,
    pub congestion_level: CongestionLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockhashWindowStatus {
    Healthy,
    Tight,
    Exhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PacketDeliveryQuality {
    Healthy,
    Constrained,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotHealth {
    Stable,
    Degraded,
    Unstable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeeMarketLevel {
    Cheap,
    Competitive,
    Expensive,
    Extreme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CongestionLevel {
    Low,
    Moderate,
    High,
    Severe,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkValidationError {
    MissingSlotTimeSamples,
    ZeroSlotTimeSample {
        sample_index: usize,
    },
    MissingPrioritizationFeeSamples,
    BlockhashValidityWindowZero,
    BlockhashContextAheadOfCurrentSlot {
        current_slot: u64,
        latest_blockhash_context_slot: u64,
    },
    DroppedPacketRateTooHigh {
        rate_bps: u16,
        max_bps: u16,
    },
    ConfirmationDelayOrderInvalid {
        p50: u16,
        p90: u16,
    },
}

impl fmt::Display for NetworkValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSlotTimeSamples => {
                write!(f, "network snapshot must include recent slot time samples")
            }
            Self::ZeroSlotTimeSample { sample_index } => {
                write!(f, "slot time sample {sample_index} must be greater than zero")
            }
            Self::MissingPrioritizationFeeSamples => write!(
                f,
                "network snapshot must include recent prioritization fee samples"
            ),
            Self::BlockhashValidityWindowZero => {
                write!(f, "blockhash validity window must be greater than zero")
            }
            Self::BlockhashContextAheadOfCurrentSlot {
                current_slot,
                latest_blockhash_context_slot,
            } => write!(
                f,
                "latest blockhash context slot {latest_blockhash_context_slot} cannot be ahead of current slot {current_slot}"
            ),
            Self::DroppedPacketRateTooHigh { rate_bps, max_bps } => write!(
                f,
                "dropped packet rate {rate_bps} bps exceeds the maximum {max_bps} bps"
            ),
            Self::ConfirmationDelayOrderInvalid { p50, p90 } => write!(
                f,
                "confirmation delay p50 ({p50}) cannot exceed p90 ({p90})"
            ),
        }
    }
}

impl std::error::Error for NetworkValidationError {}

impl NetworkSnapshot {
    pub fn validate(&self) -> Result<(), NetworkValidationError> {
        if self.recent_slot_time_samples_ms.is_empty() {
            return Err(NetworkValidationError::MissingSlotTimeSamples);
        }

        for (sample_index, slot_time_ms) in self.recent_slot_time_samples_ms.iter().enumerate() {
            if *slot_time_ms == 0 {
                return Err(NetworkValidationError::ZeroSlotTimeSample { sample_index });
            }
        }

        if self.recent_prioritization_fees_microlamports.is_empty() {
            return Err(NetworkValidationError::MissingPrioritizationFeeSamples);
        }

        if self.blockhash_validity_window_slots == 0 {
            return Err(NetworkValidationError::BlockhashValidityWindowZero);
        }

        if self.latest_blockhash_context_slot > self.current_slot {
            return Err(NetworkValidationError::BlockhashContextAheadOfCurrentSlot {
                current_slot: self.current_slot,
                latest_blockhash_context_slot: self.latest_blockhash_context_slot,
            });
        }

        if self.dropped_packet_rate_bps > MAX_DROPPED_PACKET_RATE_BPS {
            return Err(NetworkValidationError::DroppedPacketRateTooHigh {
                rate_bps: self.dropped_packet_rate_bps,
                max_bps: MAX_DROPPED_PACKET_RATE_BPS,
            });
        }

        if self.confirmation_delay_slots_p50 > self.confirmation_delay_slots_p90 {
            return Err(NetworkValidationError::ConfirmationDelayOrderInvalid {
                p50: self.confirmation_delay_slots_p50,
                p90: self.confirmation_delay_slots_p90,
            });
        }

        Ok(())
    }

    pub fn profile(&self) -> Result<NetworkProfile, NetworkValidationError> {
        self.validate()?;

        let average_slot_time_ms = average_u32(&self.recent_slot_time_samples_ms);
        let slot_time_jitter_ms = slot_time_jitter_ms(&self.recent_slot_time_samples_ms);
        let median_prioritization_fee_microlamports =
            median_u64(&self.recent_prioritization_fees_microlamports);
        let min_prioritization_fee_microlamports = self
            .recent_prioritization_fees_microlamports
            .iter()
            .copied()
            .min()
            .expect("validated fee samples cannot be empty");
        let max_prioritization_fee_microlamports = self
            .recent_prioritization_fees_microlamports
            .iter()
            .copied()
            .max()
            .expect("validated fee samples cannot be empty");
        let blockhash_age_slots = self.current_slot - self.latest_blockhash_context_slot;
        let blockhash_headroom_slots = self
            .blockhash_validity_window_slots
            .saturating_sub(blockhash_age_slots);
        let blockhash_window_status = classify_blockhash_window_status(
            blockhash_age_slots,
            self.blockhash_validity_window_slots,
        );
        let packet_delivery_quality =
            classify_packet_delivery_quality(self.dropped_packet_rate_bps);
        let slot_health = classify_slot_health(average_slot_time_ms, slot_time_jitter_ms);
        let fee_market_level =
            classify_fee_market_level(median_prioritization_fee_microlamports);
        let congestion_level = classify_congestion_level(CongestionInputs {
            average_slot_time_ms,
            median_prioritization_fee_microlamports,
            pending_transaction_estimate: self.pending_transaction_estimate,
            dropped_packet_rate_bps: self.dropped_packet_rate_bps,
            confirmation_delay_slots_p90: self.confirmation_delay_slots_p90,
        });

        Ok(NetworkProfile {
            current_slot: self.current_slot,
            average_slot_time_ms,
            slot_time_jitter_ms,
            median_prioritization_fee_microlamports,
            min_prioritization_fee_microlamports,
            max_prioritization_fee_microlamports,
            blockhash_age_slots,
            blockhash_headroom_slots,
            blockhash_window_status,
            pending_transaction_estimate: self.pending_transaction_estimate,
            dropped_packet_rate_bps: self.dropped_packet_rate_bps,
            packet_delivery_quality,
            confirmation_delay_slots_p50: self.confirmation_delay_slots_p50,
            confirmation_delay_slots_p90: self.confirmation_delay_slots_p90,
            slot_health,
            fee_market_level,
            congestion_level,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CongestionInputs {
    pub average_slot_time_ms: u32,
    pub median_prioritization_fee_microlamports: u64,
    pub pending_transaction_estimate: u32,
    pub dropped_packet_rate_bps: u16,
    pub confirmation_delay_slots_p90: u16,
}

pub fn classify_blockhash_window_status(
    blockhash_age_slots: u64,
    blockhash_validity_window_slots: u64,
) -> BlockhashWindowStatus {
    if blockhash_age_slots >= blockhash_validity_window_slots {
        return BlockhashWindowStatus::Exhausted;
    }

    let headroom_slots = blockhash_validity_window_slots - blockhash_age_slots;
    if headroom_slots <= TIGHT_BLOCKHASH_HEADROOM_SLOTS
        || headroom_slots * 100 < blockhash_validity_window_slots * 20
    {
        BlockhashWindowStatus::Tight
    } else {
        BlockhashWindowStatus::Healthy
    }
}

pub fn classify_packet_delivery_quality(dropped_packet_rate_bps: u16) -> PacketDeliveryQuality {
    match dropped_packet_rate_bps {
        0..=199 => PacketDeliveryQuality::Healthy,
        200..=999 => PacketDeliveryQuality::Constrained,
        _ => PacketDeliveryQuality::Degraded,
    }
}

pub fn classify_slot_health(average_slot_time_ms: u32, slot_time_jitter_ms: u32) -> SlotHealth {
    if average_slot_time_ms <= 500 && slot_time_jitter_ms <= 150 {
        SlotHealth::Stable
    } else if average_slot_time_ms <= 700 && slot_time_jitter_ms <= 300 {
        SlotHealth::Degraded
    } else {
        SlotHealth::Unstable
    }
}

pub fn classify_fee_market_level(
    median_prioritization_fee_microlamports: u64,
) -> FeeMarketLevel {
    match median_prioritization_fee_microlamports {
        0..=999 => FeeMarketLevel::Cheap,
        1_000..=9_999 => FeeMarketLevel::Competitive,
        10_000..=49_999 => FeeMarketLevel::Expensive,
        _ => FeeMarketLevel::Extreme,
    }
}

pub fn classify_congestion_level(inputs: CongestionInputs) -> CongestionLevel {
    let mut score = 0u8;

    score += match inputs.average_slot_time_ms {
        0..=500 => 0,
        501..=700 => 1,
        _ => 2,
    };

    score += match inputs.median_prioritization_fee_microlamports {
        0..=9_999 => 0,
        10_000..=49_999 => 1,
        _ => 2,
    };

    score += match inputs.pending_transaction_estimate {
        0..=4_999 => 0,
        5_000..=14_999 => 1,
        _ => 2,
    };

    score += match inputs.dropped_packet_rate_bps {
        0..=199 => 0,
        200..=999 => 1,
        _ => 2,
    };

    score += match inputs.confirmation_delay_slots_p90 {
        0..=6 => 0,
        7..=12 => 1,
        _ => 2,
    };

    match score {
        0..=1 => CongestionLevel::Low,
        2..=4 => CongestionLevel::Moderate,
        5..=7 => CongestionLevel::High,
        _ => CongestionLevel::Severe,
    }
}

fn average_u32(values: &[u32]) -> u32 {
    let sum: u64 = values.iter().map(|value| *value as u64).sum();
    (sum / values.len() as u64) as u32
}

fn slot_time_jitter_ms(values: &[u32]) -> u32 {
    let min = values.iter().copied().min().expect("validated slice not empty");
    let max = values.iter().copied().max().expect("validated slice not empty");
    max - min
}

fn median_u64(values: &[u64]) -> u64 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let mid = sorted.len() / 2;

    if sorted.len() % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) / 2
    } else {
        sorted[mid]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_snapshot() -> NetworkSnapshot {
        NetworkSnapshot {
            current_slot: 10_250,
            recent_slot_time_samples_ms: vec![410, 430, 450, 420, 440],
            recent_prioritization_fees_microlamports: vec![2_000, 3_000, 1_500, 2_500, 3_500],
            blockhash_validity_window_slots: 150,
            latest_blockhash_context_slot: 10_230,
            pending_transaction_estimate: 2_500,
            dropped_packet_rate_bps: 75,
            confirmation_delay_slots_p50: 2,
            confirmation_delay_slots_p90: 4,
        }
    }

    #[test]
    fn validates_and_builds_profile_for_a_healthy_network_snapshot() {
        let snapshot = sample_snapshot();

        snapshot
            .validate()
            .expect("sample snapshot should validate");
        let profile = snapshot.profile().expect("sample snapshot should profile");

        assert_eq!(profile.current_slot, 10_250);
        assert_eq!(profile.average_slot_time_ms, 430);
        assert_eq!(profile.slot_time_jitter_ms, 40);
        assert_eq!(profile.median_prioritization_fee_microlamports, 2_500);
        assert_eq!(profile.min_prioritization_fee_microlamports, 1_500);
        assert_eq!(profile.max_prioritization_fee_microlamports, 3_500);
        assert_eq!(profile.blockhash_age_slots, 20);
        assert_eq!(profile.blockhash_headroom_slots, 130);
        assert_eq!(profile.blockhash_window_status, BlockhashWindowStatus::Healthy);
        assert_eq!(profile.packet_delivery_quality, PacketDeliveryQuality::Healthy);
        assert_eq!(profile.slot_health, SlotHealth::Stable);
        assert_eq!(profile.fee_market_level, FeeMarketLevel::Competitive);
        assert_eq!(profile.congestion_level, CongestionLevel::Low);
    }

    #[test]
    fn rejects_snapshot_without_slot_time_samples() {
        let mut snapshot = sample_snapshot();
        snapshot.recent_slot_time_samples_ms.clear();

        assert_eq!(
            snapshot.validate(),
            Err(NetworkValidationError::MissingSlotTimeSamples)
        );
    }

    #[test]
    fn rejects_slot_time_samples_with_zero_values() {
        let mut snapshot = sample_snapshot();
        snapshot.recent_slot_time_samples_ms[2] = 0;

        assert_eq!(
            snapshot.validate(),
            Err(NetworkValidationError::ZeroSlotTimeSample { sample_index: 2 })
        );
    }

    #[test]
    fn rejects_snapshot_without_fee_samples() {
        let mut snapshot = sample_snapshot();
        snapshot.recent_prioritization_fees_microlamports.clear();

        assert_eq!(
            snapshot.validate(),
            Err(NetworkValidationError::MissingPrioritizationFeeSamples)
        );
    }

    #[test]
    fn rejects_zero_blockhash_validity_window() {
        let mut snapshot = sample_snapshot();
        snapshot.blockhash_validity_window_slots = 0;

        assert_eq!(
            snapshot.validate(),
            Err(NetworkValidationError::BlockhashValidityWindowZero)
        );
    }

    #[test]
    fn rejects_blockhash_context_slots_ahead_of_current_slot() {
        let mut snapshot = sample_snapshot();
        snapshot.latest_blockhash_context_slot = snapshot.current_slot + 1;

        assert_eq!(
            snapshot.validate(),
            Err(NetworkValidationError::BlockhashContextAheadOfCurrentSlot {
                current_slot: snapshot.current_slot,
                latest_blockhash_context_slot: snapshot.current_slot + 1,
            })
        );
    }

    #[test]
    fn rejects_packet_drop_rate_above_upper_bound() {
        let mut snapshot = sample_snapshot();
        snapshot.dropped_packet_rate_bps = MAX_DROPPED_PACKET_RATE_BPS + 1;

        assert_eq!(
            snapshot.validate(),
            Err(NetworkValidationError::DroppedPacketRateTooHigh {
                rate_bps: MAX_DROPPED_PACKET_RATE_BPS + 1,
                max_bps: MAX_DROPPED_PACKET_RATE_BPS,
            })
        );
    }

    #[test]
    fn accepts_packet_drop_rate_at_upper_bound() {
        let mut snapshot = sample_snapshot();
        snapshot.dropped_packet_rate_bps = MAX_DROPPED_PACKET_RATE_BPS;

        assert!(snapshot.validate().is_ok());
    }

    #[test]
    fn rejects_confirmation_delays_when_p50_exceeds_p90() {
        let mut snapshot = sample_snapshot();
        snapshot.confirmation_delay_slots_p50 = 7;
        snapshot.confirmation_delay_slots_p90 = 6;

        assert_eq!(
            snapshot.validate(),
            Err(NetworkValidationError::ConfirmationDelayOrderInvalid { p50: 7, p90: 6 })
        );
    }

    #[test]
    fn blockhash_window_status_covers_healthy_tight_and_exhausted_ranges() {
        assert_eq!(
            classify_blockhash_window_status(20, 150),
            BlockhashWindowStatus::Healthy
        );
        assert_eq!(
            classify_blockhash_window_status(131, 150),
            BlockhashWindowStatus::Tight
        );
        assert_eq!(
            classify_blockhash_window_status(150, 150),
            BlockhashWindowStatus::Exhausted
        );
        assert_eq!(
            classify_blockhash_window_status(170, 150),
            BlockhashWindowStatus::Exhausted
        );
    }

    #[test]
    fn packet_delivery_quality_covers_all_ranges() {
        assert_eq!(
            classify_packet_delivery_quality(0),
            PacketDeliveryQuality::Healthy
        );
        assert_eq!(
            classify_packet_delivery_quality(199),
            PacketDeliveryQuality::Healthy
        );
        assert_eq!(
            classify_packet_delivery_quality(200),
            PacketDeliveryQuality::Constrained
        );
        assert_eq!(
            classify_packet_delivery_quality(999),
            PacketDeliveryQuality::Constrained
        );
        assert_eq!(
            classify_packet_delivery_quality(1_000),
            PacketDeliveryQuality::Degraded
        );
    }

    #[test]
    fn slot_health_classification_covers_all_ranges() {
        assert_eq!(classify_slot_health(450, 100), SlotHealth::Stable);
        assert_eq!(classify_slot_health(650, 200), SlotHealth::Degraded);
        assert_eq!(classify_slot_health(820, 200), SlotHealth::Unstable);
        assert_eq!(classify_slot_health(650, 350), SlotHealth::Unstable);
    }

    #[test]
    fn fee_market_classification_covers_all_ranges() {
        assert_eq!(classify_fee_market_level(0), FeeMarketLevel::Cheap);
        assert_eq!(classify_fee_market_level(999), FeeMarketLevel::Cheap);
        assert_eq!(classify_fee_market_level(1_000), FeeMarketLevel::Competitive);
        assert_eq!(classify_fee_market_level(9_999), FeeMarketLevel::Competitive);
        assert_eq!(classify_fee_market_level(10_000), FeeMarketLevel::Expensive);
        assert_eq!(classify_fee_market_level(49_999), FeeMarketLevel::Expensive);
        assert_eq!(classify_fee_market_level(50_000), FeeMarketLevel::Extreme);
    }

    #[test]
    fn congestion_classification_distinguishes_low_moderate_high_and_severe() {
        assert_eq!(
            classify_congestion_level(CongestionInputs {
                average_slot_time_ms: 450,
                median_prioritization_fee_microlamports: 2_500,
                pending_transaction_estimate: 2_000,
                dropped_packet_rate_bps: 100,
                confirmation_delay_slots_p90: 4,
            }),
            CongestionLevel::Low
        );
        assert_eq!(
            classify_congestion_level(CongestionInputs {
                average_slot_time_ms: 650,
                median_prioritization_fee_microlamports: 15_000,
                pending_transaction_estimate: 3_000,
                dropped_packet_rate_bps: 100,
                confirmation_delay_slots_p90: 4,
            }),
            CongestionLevel::Moderate
        );
        assert_eq!(
            classify_congestion_level(CongestionInputs {
                average_slot_time_ms: 650,
                median_prioritization_fee_microlamports: 20_000,
                pending_transaction_estimate: 8_000,
                dropped_packet_rate_bps: 450,
                confirmation_delay_slots_p90: 8,
            }),
            CongestionLevel::High
        );
        assert_eq!(
            classify_congestion_level(CongestionInputs {
                average_slot_time_ms: 920,
                median_prioritization_fee_microlamports: 75_000,
                pending_transaction_estimate: 20_000,
                dropped_packet_rate_bps: 1_500,
                confirmation_delay_slots_p90: 14,
            }),
            CongestionLevel::Severe
        );
    }

    #[test]
    fn profile_uses_integer_median_for_even_fee_sample_sets() {
        let mut snapshot = sample_snapshot();
        snapshot.recent_prioritization_fees_microlamports = vec![1_000, 2_000, 10_000, 14_000];

        let profile = snapshot.profile().expect("snapshot should profile");

        assert_eq!(profile.median_prioritization_fee_microlamports, 6_000);
        assert_eq!(profile.fee_market_level, FeeMarketLevel::Competitive);
    }

    #[test]
    fn profile_marks_tight_blockhash_window_when_headroom_is_low() {
        let mut snapshot = sample_snapshot();
        snapshot.latest_blockhash_context_slot = snapshot.current_slot - 140;

        let profile = snapshot.profile().expect("snapshot should profile");

        assert_eq!(profile.blockhash_age_slots, 140);
        assert_eq!(profile.blockhash_headroom_slots, 10);
        assert_eq!(profile.blockhash_window_status, BlockhashWindowStatus::Tight);
    }

    #[test]
    fn profile_marks_exhausted_blockhash_window_when_age_reaches_validity_limit() {
        let mut snapshot = sample_snapshot();
        snapshot.latest_blockhash_context_slot =
            snapshot.current_slot - snapshot.blockhash_validity_window_slots;

        let profile = snapshot.profile().expect("snapshot should profile");

        assert_eq!(profile.blockhash_headroom_slots, 0);
        assert_eq!(
            profile.blockhash_window_status,
            BlockhashWindowStatus::Exhausted
        );
    }

    #[test]
    fn profile_can_represent_degraded_packet_delivery_without_failing_validation() {
        let mut snapshot = sample_snapshot();
        snapshot.dropped_packet_rate_bps = 1_700;

        let profile = snapshot.profile().expect("snapshot should profile");

        assert_eq!(profile.packet_delivery_quality, PacketDeliveryQuality::Degraded);
        assert!(matches!(
            profile.congestion_level,
            CongestionLevel::Moderate | CongestionLevel::High | CongestionLevel::Severe
        ));
    }

    #[test]
    fn current_slot_zero_is_allowed_when_context_is_consistent() {
        let snapshot = NetworkSnapshot {
            current_slot: 0,
            recent_slot_time_samples_ms: vec![500, 510, 495],
            recent_prioritization_fees_microlamports: vec![0, 100, 300],
            blockhash_validity_window_slots: 150,
            latest_blockhash_context_slot: 0,
            pending_transaction_estimate: 0,
            dropped_packet_rate_bps: 0,
            confirmation_delay_slots_p50: 0,
            confirmation_delay_slots_p90: 0,
        };

        let profile = snapshot.profile().expect("snapshot should profile");

        assert_eq!(profile.blockhash_age_slots, 0);
        assert_eq!(profile.blockhash_window_status, BlockhashWindowStatus::Healthy);
        assert_eq!(profile.fee_market_level, FeeMarketLevel::Cheap);
    }
}
