/// Baked, initial data availability fees parameters.
/// Key prefix: 0x01
/// ------------------------------------------------------------

// Call entry per calldata byte da fee key & value.
pub const CALL_ENTRY_PER_CALLLDATA_BYTE_DA_FEE_KEY: [u8; 2] = [0x01, 0x00];
pub const CALL_ENTRY_PER_CALLLDATA_BYTE_DA_FEE_VALUE: u64 = 1;

// Liftup entry per spent lift txo per scriptsig byte da fee key & value.
pub const LIFTUP_ENTRY_PER_SPENT_LIFT_TXO_PER_SCRIPTSIG_BYTE_DA_FEE_KEY: [u8; 2] = [0x01, 0x01];
pub const LIFTUP_ENTRY_PER_SPENT_LIFT_TXO_PER_SCRIPTSIG_BYTE_DA_FEE_VALUE: u64 = 4;

// Liftup entry per spent lift txo per witness byte da fee key & value.
pub const LIFTUP_ENTRY_PER_SPENT_LIFT_TXO_PER_WITNESS_BYTE_DA_FEE_KEY: [u8; 2] = [0x01, 0x02];
pub const LIFTUP_ENTRY_PER_SPENT_LIFT_TXO_PER_WITNESS_BYTE_DA_FEE_VALUE: u64 = 1;

// Swapout entry per scriptpubkey byte da fee key & value.
pub const SWAPOUT_ENTRY_PER_SCRIPTPUBKEY_BYTE_DA_FEE_KEY: [u8; 2] = [0x01, 0x03];
pub const SWAPOUT_ENTRY_PER_SCRIPTPUBKEY_BYTE_DA_FEE_VALUE: u64 = 4;

// Deploy entry per compiled contract byte da fee key & value.
pub const DEPLOY_ENTRY_PER_COMPILED_CONTRACT_BYTE_DA_FEE_KEY: [u8; 2] = [0x01, 0x04];
pub const DEPLOY_ENTRY_PER_COMPILED_CONTRACT_BYTE_DA_FEE_VALUE: u64 = 10; // Intentionally overpriced to discourage spam.

// Config entry per secondary aggregation key byte da fee key & value.
pub const CONFIG_ENTRY_PER_SECONDARY_AGGREGATION_KEY_BYTE_DA_FEE_KEY: [u8; 2] = [0x01, 0x05];
pub const CONFIG_ENTRY_PER_SECONDARY_AGGREGATION_KEY_BYTE_DA_FEE_VALUE: u64 = 1;

// Config entry per flame config byte da fee key & value.
pub const CONFIG_ENTRY_PER_FLAME_CONFIG_BYTE_DA_FEE_KEY: [u8; 2] = [0x01, 0x06];
pub const CONFIG_ENTRY_PER_FLAME_CONFIG_BYTE_DA_FEE_VALUE: u64 = 1;
