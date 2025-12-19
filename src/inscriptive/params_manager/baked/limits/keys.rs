/// PREFIX BYTE: 0x01.

/// Liftup limits
/// ------------------------------------------------------------
pub const MIN_LIFTUP_UTXO_COUNT_KEY: [u8; 2] = [0x01, 0x00];
pub const MAX_LIFTUP_UTXO_COUNT_KEY: [u8; 2] = [0x01, 0x01];

pub const MIN_LIFTUP_PER_LIFTED_UTXO_AMOUNT_KEY: [u8; 2] = [0x01, 0x02];
pub const MAX_LIFTUP_PER_LIFTED_UTXO_AMOUNT_KEY: [u8; 2] = [0x01, 0x03];

pub const MIN_LIFTUP_PER_LIFTED_UTXO_WITNESS_BYTE_SIZE_KEY: [u8; 2] = [0x01, 0x04];
pub const MAX_LIFTUP_PER_LIFTED_UTXO_WITNESS_BYTE_SIZE_KEY: [u8; 2] = [0x01, 0x05];
