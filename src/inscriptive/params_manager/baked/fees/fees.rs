/// Various fee tiers in satoshis.
/// ------------------------------------------------------------
// Initial onboarding fee for registering a new account with various managers.
pub const ACCOUNT_ONBOARDING_FEE: u64 = 1_000;

// Initial deployment fee for deploying a new contract with various managers.
pub const CONTRACT_DEPLOYMENT_FEE: u64 = 100_000;

// Base fee for a move entry.
pub const MOVE_BASE_FEE: u64 = 50;
// Dynamic PPM fee for a move entry.
pub const MOVE_LIQUIDITY_PPM_FEE: u64 = 100;

// Base fee for a call entry.
pub const CALL_BASE_FEE: u64 = 100;
// Data availability fee per byte of calldata.
// Currently set to 10 satoshis per byte.
pub const CALLDATA_PER_BYTE_FEE: u64 = 10;
// Dynamic PPM fee for the sum of all in-call liquidity movements (transfers & shadow allocations).
pub const IN_CALL_OVERALL_LIQUIDITY_PPM_FEE: u64 = 100;

// Base fee for adding liquidity to the engine.
pub const ADD_LIQUIDITY_BASE_FEE: u64 = 50;

// Base fee for removing liquidity from the engine.
pub const SUB_LIQUIDITY_BASE_FEE: u64 = 1_000;
// Dynamic PPM fee for removing liquidity from the engine.
pub const SUB_LIQUIDITY_PPM_FEE: u64 = 10;

// Base fee for the liftup entry.
pub const LIFTUP_BASE_FEE: u64 = 50;
// Dynamic PPM fee for a liftup liquidity movement.
pub const LIFTUP_LIQUIDITY_PPM_FEE: u64 = 0;
// Base fee for lifting up each previous transaction output.
pub const LIFTUP_PER_LIFTED_UTXO_BASE_FEE: u64 = 100;
// Data availability fee per byte of witness data for each lifted previous transaction output.
pub const LIFTUP_PER_LIFTED_UTXO_WITNESS_PER_BYTE_FEE: u64 = 0;

// Base fee for the swapout entry.
pub const SWAPOUT_BASE_FEE: u64 = 1_000;
// Data availability fee per byte of scriptPubKey for the swapout entry.
pub const SWAPOUT_SPK_PER_BYTE_FEE: u64 = 10;
// Dynamic PPM fee for a swapout liquidity movement.
pub const SWAPOUT_LIQUIDITY_PPM_FEE: u64 = 100;

// Base fee for an initial config entry that sets BLS key.
// NOTE: DoS is not possible; one-time-only-event, and the ACCOUNT_ONBOARDING_FEE gives more than enough cost coverage.
pub const CONFIG_INIT_BASE_FEE: u64 = 0;
// Base fee for subsequent config entries that update the secondary aggregation key.
pub const CONFIG_UPDATE_BASE_FEE: u64 = 50;
// Data availability fee per byte of the secondary aggregation key for subsequent config entries.
pub const CONFIG_PER_SECONDARY_AGGREGATION_KEY_BYTE_FEE: u64 = 10;
// Data availability fee per byte of the BLS key for subsequent config entries.
pub const CONFIG_PER_BLS_KEY_BYTE_FEE: u64 = 10;

// Base fee for a nop entry.
// NOTE: Current not supported; reserved for future upgrades.
pub const NOP_BASE_FEE: u64 = 0;

// Base fee for a fail entry.
// NOTE: Current not supported; reserved for future upgrades.
pub const FAIL_BASE_FEE: u64 = 0;
