/// Baked, initial base fees parameters.
/// Key prefix: 0x00
/// ------------------------------------------------------------

// Move entry base fee key & value.
pub const MOVE_ENTRY_BASE_FEE_KEY: [u8; 2] = [0x00, 0x00];
pub const MOVE_ENTRY_BASE_FEE_VALUE: u64 = 10;

// Call entry base fee key & value.
pub const CALL_ENTRY_BASE_FEE_KEY: [u8; 2] = [0x00, 0x01];
pub const CALL_ENTRY_BASE_FEE_VALUE: u64 = 20;

// Add entry base fee key & value.
pub const ADD_ENTRY_BASE_FEE_KEY: [u8; 2] = [0x00, 0x02];
pub const ADD_ENTRY_BASE_FEE_VALUE: u64 = 10;

// Sub entry base fee key & value.
pub const SUB_ENTRY_BASE_FEE_KEY: [u8; 2] = [0x00, 0x03];
pub const SUB_ENTRY_BASE_FEE_VALUE: u64 = 10;

// Liftup entry base fee key & value.
pub const LIFTUP_ENTRY_BASE_FEE_KEY: [u8; 2] = [0x00, 0x04];
pub const LIFTUP_ENTRY_BASE_FEE_VALUE: u64 = 10;

// In-Liftup per spent lift txo base fee key & value.
pub const IN_LIFTUP_PER_SPENT_LIFT_TXO_BASE_FEE_KEY: [u8; 2] = [0x00, 0x05];
pub const IN_LIFTUP_PER_SPENT_LIFT_TXO_BASE_FEE_VALUE: u64 = 50;

// Swapout entry base fee key & value.
pub const SWAPOUT_ENTRY_BASE_FEE_KEY: [u8; 2] = [0x00, 0x06];
pub const SWAPOUT_ENTRY_BASE_FEE_VALUE: u64 = 50;

// Deploy entry base fee key & value.
pub const DEPLOY_ENTRY_BASE_FEE_KEY: [u8; 2] = [0x00, 0x07];
pub const DEPLOY_ENTRY_BASE_FEE_VALUE: u64 = 50;

// Config entry base fee key & value.
pub const CONFIG_ENTRY_BASE_FEE_KEY: [u8; 2] = [0x00, 0x08];
pub const CONFIG_ENTRY_BASE_FEE_VALUE: u64 = 10;

// Nop entry base fee key & value.
pub const NOP_ENTRY_BASE_FEE_KEY: [u8; 2] = [0x00, 0x09];
pub const NOP_ENTRY_BASE_FEE_VALUE: u64 = 0;

// Fail entry base fee key & value.
pub const FAIL_ENTRY_BASE_FEE_KEY: [u8; 2] = [0x00, 0x0A];
pub const FAIL_ENTRY_BASE_FEE_VALUE: u64 = 0;
