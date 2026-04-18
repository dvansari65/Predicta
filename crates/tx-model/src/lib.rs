use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

pub const MAX_TRANSACTION_SIZE_BYTES: u16 = 1232;
pub const MAX_COMPUTE_UNIT_LIMIT: u32 = 1_400_000;
pub const STALE_BLOCKHASH_AGE_SLOTS: u64 = 150;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountMeta {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instruction {
    pub program_id: String,
    pub accounts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    pub instructions: Vec<Instruction>,
    pub accounts: Vec<AccountMeta>,
    pub compute_unit_limit: u32,
    pub priority_fee_microlamports: u64,
    pub tx_size_bytes: u16,
    pub recent_blockhash_age_slots: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionProfile {
    pub instruction_count: usize,
    pub signer_count: usize,
    pub writable_account_count: usize,
    pub readonly_account_count: usize,
    pub writable_signer_count: usize,
    pub readonly_signer_count: usize,
    pub nonsigner_writable_account_count: usize,
    pub nonsigner_readonly_account_count: usize,
    pub compute_unit_limit: u32,
    pub priority_fee_microlamports: u64,
    pub tx_size_bytes: u16,
    pub recent_blockhash_age_slots: u64,
    pub blockhash_freshness: BlockhashFreshness,
    pub unique_program_ids: Vec<String>,
    pub writable_accounts: Vec<String>,
    pub readonly_accounts: Vec<String>,
    pub signer_accounts: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockhashFreshness {
    Fresh,
    Aging,
    Stale,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionValidationError {
    MissingInstructions,
    MissingAccounts,
    InvalidProgramId {
        instruction_index: usize,
    },
    InvalidAccountPubkey {
        account_index: usize,
    },
    DuplicateAccountPubkey {
        pubkey: String,
    },
    InstructionReferencesUnknownAccount {
        instruction_index: usize,
        account_pubkey: String,
    },
    ComputeUnitLimitZero,
    ComputeUnitLimitTooHigh {
        limit: u32,
        max: u32,
    },
    TransactionSizeZero,
    TransactionSizeTooHigh {
        size: u16,
        max: u16,
    },
}

impl fmt::Display for TransactionValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingInstructions => write!(f, "transaction must include at least one instruction"),
            Self::MissingAccounts => write!(f, "transaction must include at least one account"),
            Self::InvalidProgramId { instruction_index } => {
                write!(f, "instruction {instruction_index} has an empty program id")
            }
            Self::InvalidAccountPubkey { account_index } => {
                write!(f, "account {account_index} has an empty public key")
            }
            Self::DuplicateAccountPubkey { pubkey } => {
                write!(f, "duplicate account public key found: {pubkey}")
            }
            Self::InstructionReferencesUnknownAccount {
                instruction_index,
                account_pubkey,
            } => write!(
                f,
                "instruction {instruction_index} references unknown account {account_pubkey}"
            ),
            Self::ComputeUnitLimitZero => write!(f, "compute unit limit must be greater than zero"),
            Self::ComputeUnitLimitTooHigh { limit, max } => {
                write!(f, "compute unit limit {limit} exceeds max supported value {max}")
            }
            Self::TransactionSizeZero => write!(f, "transaction size must be greater than zero"),
            Self::TransactionSizeTooHigh { size, max } => {
                write!(f, "transaction size {size} exceeds max packet size {max}")
            }
        }
    }
}

impl std::error::Error for TransactionValidationError {}

impl Transaction {
    pub fn validate(&self) -> Result<(), TransactionValidationError> {
        if self.instructions.is_empty() {
            return Err(TransactionValidationError::MissingInstructions);
        }

        if self.accounts.is_empty() {
            return Err(TransactionValidationError::MissingAccounts);
        }

        if self.compute_unit_limit == 0 {
            return Err(TransactionValidationError::ComputeUnitLimitZero);
        }

        if self.compute_unit_limit > MAX_COMPUTE_UNIT_LIMIT {
            return Err(TransactionValidationError::ComputeUnitLimitTooHigh {
                limit: self.compute_unit_limit,
                max: MAX_COMPUTE_UNIT_LIMIT,
            });
        }

        if self.tx_size_bytes == 0 {
            return Err(TransactionValidationError::TransactionSizeZero);
        }

        if self.tx_size_bytes > MAX_TRANSACTION_SIZE_BYTES {
            return Err(TransactionValidationError::TransactionSizeTooHigh {
                size: self.tx_size_bytes,
                max: MAX_TRANSACTION_SIZE_BYTES,
            });
        }

        let mut account_keys = BTreeSet::new();
        for (account_index, account) in self.accounts.iter().enumerate() {
            let pubkey = account.pubkey.trim();
            if pubkey.is_empty() {
                return Err(TransactionValidationError::InvalidAccountPubkey { account_index });
            }

            if !account_keys.insert(pubkey.to_owned()) {
                return Err(TransactionValidationError::DuplicateAccountPubkey {
                    pubkey: pubkey.to_owned(),
                });
            }
        }

        for (instruction_index, instruction) in self.instructions.iter().enumerate() {
            if instruction.program_id.trim().is_empty() {
                return Err(TransactionValidationError::InvalidProgramId { instruction_index });
            }

            for account_pubkey in &instruction.accounts {
                let account_pubkey = account_pubkey.trim();
                if !account_keys.contains(account_pubkey) {
                    return Err(TransactionValidationError::InstructionReferencesUnknownAccount {
                        instruction_index,
                        account_pubkey: account_pubkey.to_owned(),
                    });
                }
            }
        }

        Ok(())
    }

    pub fn profile(&self) -> Result<TransactionProfile, TransactionValidationError> {
        self.validate()?;

        let signer_accounts = sorted_unique(
            self.accounts
                .iter()
                .filter(|account| account.is_signer)
                .map(|account| account.pubkey.trim()),
        );
        let writable_accounts = sorted_unique(
            self.accounts
                .iter()
                .filter(|account| account.is_writable)
                .map(|account| account.pubkey.trim()),
        );
        let readonly_accounts = sorted_unique(
            self.accounts
                .iter()
                .filter(|account| !account.is_writable)
                .map(|account| account.pubkey.trim()),
        );
        let unique_program_ids = sorted_unique(
            self.instructions
                .iter()
                .map(|instruction| instruction.program_id.trim()),
        );

        let signer_count = signer_accounts.len();
        let writable_account_count = writable_accounts.len();
        let readonly_account_count = readonly_accounts.len();
        let writable_signer_count = self
            .accounts
            .iter()
            .filter(|account| account.is_signer && account.is_writable)
            .count();
        let readonly_signer_count = self
            .accounts
            .iter()
            .filter(|account| account.is_signer && !account.is_writable)
            .count();
        let nonsigner_writable_account_count = self
            .accounts
            .iter()
            .filter(|account| !account.is_signer && account.is_writable)
            .count();
        let nonsigner_readonly_account_count = self
            .accounts
            .iter()
            .filter(|account| !account.is_signer && !account.is_writable)
            .count();

        Ok(TransactionProfile {
            instruction_count: self.instructions.len(),
            signer_count,
            writable_account_count,
            readonly_account_count,
            writable_signer_count,
            readonly_signer_count,
            nonsigner_writable_account_count,
            nonsigner_readonly_account_count,
            compute_unit_limit: self.compute_unit_limit,
            priority_fee_microlamports: self.priority_fee_microlamports,
            tx_size_bytes: self.tx_size_bytes,
            recent_blockhash_age_slots: self.recent_blockhash_age_slots,
            blockhash_freshness: classify_blockhash_age(self.recent_blockhash_age_slots),
            unique_program_ids,
            writable_accounts,
            readonly_accounts,
            signer_accounts,
        })
    }
}

pub fn classify_blockhash_age(age_slots: u64) -> BlockhashFreshness {
    match age_slots {
        0..=49 => BlockhashFreshness::Fresh,
        50..=STALE_BLOCKHASH_AGE_SLOTS => BlockhashFreshness::Aging,
        _ => BlockhashFreshness::Stale,
    }
}

fn sorted_unique<'a>(values: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    values
        .into_iter()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_transaction() -> Transaction {
        Transaction {
            instructions: vec![
                Instruction {
                    program_id: "ComputeBudget111111111111111111111111111111".to_owned(),
                    accounts: vec!["payer".to_owned()],
                },
                Instruction {
                    program_id: "JupiterSwap11111111111111111111111111111111".to_owned(),
                    accounts: vec![
                        "payer".to_owned(),
                        "user_token".to_owned(),
                        "pool".to_owned(),
                    ],
                },
            ],
            accounts: vec![
                AccountMeta {
                    pubkey: "payer".to_owned(),
                    is_signer: true,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: "user_token".to_owned(),
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: "pool".to_owned(),
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: "system_program".to_owned(),
                    is_signer: false,
                    is_writable: false,
                },
            ],
            compute_unit_limit: 250_000,
            priority_fee_microlamports: 5_000,
            tx_size_bytes: 420,
            recent_blockhash_age_slots: 12,
        }
    }

    #[test]
    fn validates_and_builds_profile_for_a_well_formed_transaction() {
        let tx = sample_transaction();

        tx.validate().expect("sample transaction should validate");
        let profile = tx.profile().expect("sample transaction should profile");

        assert_eq!(profile.instruction_count, 2);
        assert_eq!(profile.signer_count, 1);
        assert_eq!(profile.writable_account_count, 2);
        assert_eq!(profile.readonly_account_count, 2);
        assert_eq!(profile.writable_signer_count, 1);
        assert_eq!(profile.readonly_signer_count, 0);
        assert_eq!(profile.nonsigner_writable_account_count, 1);
        assert_eq!(profile.nonsigner_readonly_account_count, 2);
        assert_eq!(profile.compute_unit_limit, 250_000);
        assert_eq!(profile.priority_fee_microlamports, 5_000);
        assert_eq!(profile.tx_size_bytes, 420);
        assert_eq!(profile.recent_blockhash_age_slots, 12);
        assert_eq!(profile.blockhash_freshness, BlockhashFreshness::Fresh);
        assert_eq!(
            profile.unique_program_ids,
            vec![
                "ComputeBudget111111111111111111111111111111".to_owned(),
                "JupiterSwap11111111111111111111111111111111".to_owned()
            ]
        );
        assert_eq!(
            profile.writable_accounts,
            vec!["payer".to_owned(), "user_token".to_owned()]
        );
        assert_eq!(
            profile.readonly_accounts,
            vec!["pool".to_owned(), "system_program".to_owned()]
        );
        assert_eq!(profile.signer_accounts, vec!["payer".to_owned()]);
    }

    #[test]
    fn rejects_transaction_without_instructions() {
        let mut tx = sample_transaction();
        tx.instructions.clear();

        assert_eq!(
            tx.validate(),
            Err(TransactionValidationError::MissingInstructions)
        );
    }

    #[test]
    fn rejects_transaction_without_accounts() {
        let mut tx = sample_transaction();
        tx.accounts.clear();

        assert_eq!(tx.validate(), Err(TransactionValidationError::MissingAccounts));
    }

    #[test]
    fn rejects_zero_compute_unit_limit() {
        let mut tx = sample_transaction();
        tx.compute_unit_limit = 0;

        assert_eq!(
            tx.validate(),
            Err(TransactionValidationError::ComputeUnitLimitZero)
        );
    }

    #[test]
    fn rejects_compute_unit_limit_above_supported_max() {
        let mut tx = sample_transaction();
        tx.compute_unit_limit = MAX_COMPUTE_UNIT_LIMIT + 1;

        assert_eq!(
            tx.validate(),
            Err(TransactionValidationError::ComputeUnitLimitTooHigh {
                limit: MAX_COMPUTE_UNIT_LIMIT + 1,
                max: MAX_COMPUTE_UNIT_LIMIT,
            })
        );
    }

    #[test]
    fn accepts_compute_unit_limit_at_supported_max() {
        let mut tx = sample_transaction();
        tx.compute_unit_limit = MAX_COMPUTE_UNIT_LIMIT;

        assert!(tx.validate().is_ok());
    }

    #[test]
    fn rejects_zero_transaction_size() {
        let mut tx = sample_transaction();
        tx.tx_size_bytes = 0;

        assert_eq!(
            tx.validate(),
            Err(TransactionValidationError::TransactionSizeZero)
        );
    }

    #[test]
    fn rejects_transaction_size_above_packet_limit() {
        let mut tx = sample_transaction();
        tx.tx_size_bytes = MAX_TRANSACTION_SIZE_BYTES + 1;

        assert_eq!(
            tx.validate(),
            Err(TransactionValidationError::TransactionSizeTooHigh {
                size: MAX_TRANSACTION_SIZE_BYTES + 1,
                max: MAX_TRANSACTION_SIZE_BYTES,
            })
        );
    }

    #[test]
    fn accepts_transaction_size_at_packet_limit() {
        let mut tx = sample_transaction();
        tx.tx_size_bytes = MAX_TRANSACTION_SIZE_BYTES;

        assert!(tx.validate().is_ok());
    }

    #[test]
    fn rejects_blank_program_id_even_when_whitespace_only() {
        let mut tx = sample_transaction();
        tx.instructions[0].program_id = "   ".to_owned();

        assert_eq!(
            tx.validate(),
            Err(TransactionValidationError::InvalidProgramId {
                instruction_index: 0,
            })
        );
    }

    #[test]
    fn rejects_blank_account_pubkey_even_when_whitespace_only() {
        let mut tx = sample_transaction();
        tx.accounts[1].pubkey = " \n\t ".to_owned();

        assert_eq!(
            tx.validate(),
            Err(TransactionValidationError::InvalidAccountPubkey { account_index: 1 })
        );
    }

    #[test]
    fn rejects_duplicate_account_pubkeys_after_trimming() {
        let mut tx = sample_transaction();
        tx.accounts[1].pubkey = " payer ".to_owned();

        assert_eq!(
            tx.validate(),
            Err(TransactionValidationError::DuplicateAccountPubkey {
                pubkey: "payer".to_owned(),
            })
        );
    }

    #[test]
    fn rejects_instruction_that_references_unknown_account() {
        let mut tx = sample_transaction();
        tx.instructions[1].accounts.push("missing_account".to_owned());

        assert_eq!(
            tx.validate(),
            Err(TransactionValidationError::InstructionReferencesUnknownAccount {
                instruction_index: 1,
                account_pubkey: "missing_account".to_owned(),
            })
        );
    }

    #[test]
    fn profile_trims_and_sorts_values_into_stable_normalized_order() {
        let tx = Transaction {
            instructions: vec![
                Instruction {
                    program_id: " zebra_program ".to_owned(),
                    accounts: vec![" beta ".to_owned(), "alpha".to_owned()],
                },
                Instruction {
                    program_id: "alpha_program".to_owned(),
                    accounts: vec!["alpha".to_owned(), " gamma".to_owned()],
                },
                Instruction {
                    program_id: "alpha_program".to_owned(),
                    accounts: vec!["alpha".to_owned()],
                },
            ],
            accounts: vec![
                AccountMeta {
                    pubkey: " gamma ".to_owned(),
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: "beta".to_owned(),
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: "alpha ".to_owned(),
                    is_signer: true,
                    is_writable: true,
                },
            ],
            compute_unit_limit: 300_000,
            priority_fee_microlamports: 1,
            tx_size_bytes: 512,
            recent_blockhash_age_slots: 80,
        };

        let profile = tx.profile().expect("transaction should normalize");

        assert_eq!(
            profile.unique_program_ids,
            vec!["alpha_program".to_owned(), "zebra_program".to_owned()]
        );
        assert_eq!(
            profile.writable_accounts,
            vec!["alpha".to_owned(), "beta".to_owned()]
        );
        assert_eq!(profile.readonly_accounts, vec!["gamma".to_owned()]);
        assert_eq!(profile.signer_accounts, vec!["alpha".to_owned()]);
        assert_eq!(profile.blockhash_freshness, BlockhashFreshness::Aging);
    }

    #[test]
    fn blockhash_age_is_classified_across_all_ranges() {
        assert_eq!(classify_blockhash_age(0), BlockhashFreshness::Fresh);
        assert_eq!(classify_blockhash_age(49), BlockhashFreshness::Fresh);
        assert_eq!(classify_blockhash_age(50), BlockhashFreshness::Aging);
        assert_eq!(
            classify_blockhash_age(STALE_BLOCKHASH_AGE_SLOTS),
            BlockhashFreshness::Aging
        );
        assert_eq!(
            classify_blockhash_age(STALE_BLOCKHASH_AGE_SLOTS + 1),
            BlockhashFreshness::Stale
        );
    }

    #[test]
    fn readonly_signers_are_counted_correctly() {
        let mut tx = sample_transaction();
        tx.accounts.push(AccountMeta {
            pubkey: "readonly_signer".to_owned(),
            is_signer: true,
            is_writable: false,
        });
        tx.instructions[1]
            .accounts
            .push("readonly_signer".to_owned());

        let profile = tx.profile().expect("transaction should profile");

        assert_eq!(profile.signer_count, 2);
        assert_eq!(profile.writable_signer_count, 1);
        assert_eq!(profile.readonly_signer_count, 1);
        assert_eq!(profile.nonsigner_readonly_account_count, 2);
    }

    #[test]
    fn profile_preserves_zero_priority_fee_as_valid_input() {
        let mut tx = sample_transaction();
        tx.priority_fee_microlamports = 0;

        let profile = tx.profile().expect("zero priority fee should be valid");

        assert_eq!(profile.priority_fee_microlamports, 0);
    }

    #[test]
    fn stale_blockhash_is_supported_but_flagged_in_profile() {
        let mut tx = sample_transaction();
        tx.recent_blockhash_age_slots = STALE_BLOCKHASH_AGE_SLOTS + 25;

        let profile = tx.profile().expect("stale blockhash should still profile");

        assert_eq!(profile.blockhash_freshness, BlockhashFreshness::Stale);
        assert_eq!(
            profile.recent_blockhash_age_slots,
            STALE_BLOCKHASH_AGE_SLOTS + 25
        );
    }
}
