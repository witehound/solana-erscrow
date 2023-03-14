use solana_program::{pubkey, pubkey::Pubkey};

pub const FACTORY_SEED: &str = "factoryinitone";
pub const ESCROW_SEED: &str = "escrowinitone";
pub const SIX_MONTHS: i64 = 300;
pub const MIN_AMOUNT: u64 = 1000000000;
pub const COMMISION_RATE: u64 = 5;
pub const COMMISSION_PUBKEY: Pubkey = pubkey!("BpvinfQbUZ7HbxnLvFYGvWG1hgqHUL6gQP5REKi5LcJi");

// let six_weeks: i64 = 6 * 2629743;
