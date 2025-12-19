/// Liftup limits
/// ------------------------------------------------------------
// Minimum liftup utxo count key.
pub const MIN_LIFTUP_UTXO_COUNT: u32 = 1;
// Maximum liftup utxo count key.
pub const MAX_LIFTUP_UTXO_COUNT: u32 = 64;

// Minimum liftup per lifted utxo amount key.
pub const MIN_LIFTUP_PER_LIFTED_UTXO_AMOUNT: u64 = 500;
// Maximum liftup per lifted utxo amount key.
pub const MAX_LIFTUP_PER_LIFTED_UTXO_AMOUNT: u64 = 1_000_000_000_000;

// Minimum liftup per lifted utxo witness byte size key.
pub const MIN_LIFTUP_PER_LIFTED_UTXO_WITNESS_BYTE_SIZE: u32 = 1;
// Maximum liftup per lifted utxo witness byte size key.
pub const MAX_LIFTUP_PER_LIFTED_UTXO_WITNESS_BYTE_SIZE: u32 = 1024 * 32;
