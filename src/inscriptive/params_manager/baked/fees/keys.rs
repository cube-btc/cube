/// PREFIX BYTE: 0x00.

pub const ACCOUNT_ONBOARDING_FEE_KEY: [u8; 2] = [0x00, 0x00];

pub const CONTRACT_DEPLOYMENT_FEE_KEY: [u8; 2] = [0x00, 0x01];

pub const MOVE_BASE_FEE_KEY: [u8; 2] = [0x00, 0x02];
pub const MOVE_LIQUIDITY_PPM_FEE_KEY: [u8; 2] = [0x00, 0x03];

pub const CALL_BASE_FEE_KEY: [u8; 2] = [0x00, 0x04];
pub const CALLDATA_PER_BYTE_FEE_KEY: [u8; 2] = [0x00, 0x05];
pub const IN_CALL_OVERALL_LIQUIDITY_PPM_FEE_KEY: [u8; 2] = [0x00, 0x06];

pub const ADD_LIQUIDITY_BASE_FEE_KEY: [u8; 2] = [0x00, 0x07];

pub const SUB_LIQUIDITY_BASE_FEE_KEY: [u8; 2] = [0x00, 0x08];
pub const SUB_LIQUIDITY_PPM_FEE_KEY: [u8; 2] = [0x00, 0x09];

pub const LIFTUP_BASE_FEE_KEY: [u8; 2] = [0x00, 0x0A];
pub const LIFTUP_LIQUIDITY_PPM_FEE_KEY: [u8; 2] = [0x00, 0x0B];
pub const LIFTUP_PER_LIFTED_UTXO_BASE_FEE_KEY: [u8; 2] = [0x00, 0x0C];
pub const LIFTUP_PER_LIFTED_UTXO_WITNESS_PER_BYTE_FEE_KEY: [u8; 2] = [0x00, 0x0D];

pub const SWAPOUT_BASE_FEE_KEY: [u8; 2] = [0x00, 0x0E];
pub const SWAPOUT_SPK_PER_BYTE_FEE_KEY: [u8; 2] = [0x00, 0x0F];
pub const SWAPOUT_LIQUIDITY_PPM_FEE_KEY: [u8; 2] = [0x00, 0x10];

pub const CONFIG_INIT_BASE_FEE_KEY: [u8; 2] = [0x00, 0x11];
pub const CONFIG_UPDATE_BASE_FEE_KEY: [u8; 2] = [0x00, 0x12];
pub const CONFIG_PER_BLS_KEY_BYTE_FEE_KEY: [u8; 2] = [0x00, 0x13];
pub const CONFIG_PER_SECONDARY_AGGREGATION_KEY_BYTE_FEE_KEY: [u8; 2] = [0x00, 0x14];

pub const NOP_BASE_FEE_KEY: [u8; 2] = [0x00, 0x15];
pub const FAIL_BASE_FEE_KEY: [u8; 2] = [0x00, 0x16];
