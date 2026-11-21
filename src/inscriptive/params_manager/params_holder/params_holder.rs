use super::opcode_ops_params::OpcodeOpsParams;

/// The period of time after which a Registered and Configured Pleb account is considered inactive and whose termination process will be initiated from that point.
/// Initially set to 2 years. Subject to change via The Governance Contract.
const CONFIGURED_PLEB_INACTIVITY_TERMINATION_PERIOD_SECONDS_DEFAULT: u64 = 63_072_000;

/// The period of time after which a RegisteredButUnconfigured Pleb account is considered inactive and whose termination process will be initiated from that point.
/// Initially set to 8 years. Subject to change via The Governance Contract.
const UNCONFIGURED_PLEB_INACTIVITY_TERMINATION_PERIOD_SECONDS_DEFAULT: u64 = 252_455_616;

/// The minimum possible value which a Pleb (Configured or Not Configured) inactivity period can be set to.
/// Set to 1 month. The Governance Contract can never set the inactivity period below this value.
pub const PLEB_INACTIVITY_PERIOD_FLOOR: u64 = 2_592_000;

/// The vitality minus zero balance value at which a mutable contract's termination process will be initiated.
/// Initially set to minus 10,000 satoshis. Subject to change via The Governance Contract.
const MUTABLE_CONTRACT_VITALITY_SUCKED_TERMINATION_VALUE_DEFAULT: i64 = -10_000;

/// The least-negative allowed vitality sucked termination value.
/// Set to minus 100 satoshis. The Governance Contract can never set the vitality sucked termination value above this value.
pub const MUTABLE_CONTRACT_VITALITY_SUCKED_TERMINATION_VALUE_FLOOR: i64 = -100;

/// The most-negative allowed vitality sucked termination value.
/// Set to minus 1,000,000,000 satoshis. The Governance Contract can never set the vitality sucked termination value below this value.
pub const MUTABLE_CONTRACT_VITALITY_SUCKED_TERMINATION_VALUE_CEILING: i64 = -1_000_000_000;

/// The default contract binsize PPM tax rate.
/// Initially set to 0.1%, meaning a contract is charged (taxed) 1 satoshi per 1_000 bytes of contract bytecode/binary size.
/// Subject to change via The Governance Contract.
const CONTRACT_BINSIZE_PPM_TAX_RATE_DEFAULT: u64 = 1_000;

/// The minimum possible value which a contract binsize PPM tax rate can be set to.
/// Set to 0.0001%, meaning a contract is charged (taxed) 1 satoshi per 100_000_000 bytes of contract bytecode/binary size.
pub const CONTRACT_BINSIZE_PPM_TAX_RATE_FLOOR: u64 = 1;

/// The maximum possible value which a contract binsize PPM tax rate can be set to.
/// Set to 10,000%, meaning a contract is charged (taxed) 100 satoshis per one byte of contract bytecode/binary size.
pub const CONTRACT_BINSIZE_PPM_TAX_RATE_CEILING: u64 = 100_000_000;

/// The default contract statesize PPM tax rate.
/// Initially set to 0.1%, meaning a contract is charged (taxed) 1 satoshi per 1_000 bytes of contract state (storage).
/// Subject to change via The Governance Contract.
const CONTRACT_STATESIZE_PPM_TAX_RATE_DEFAULT: u64 = 1_000;

/// The minimum possible value which a contract statesize PPM tax rate can be set to.
/// Set to 0.0001%, meaning a contract is charged (taxed) 1 satoshi per 100_000_000 bytes of contract state size (storage).
pub const CONTRACT_STATESIZE_PPM_TAX_RATE_FLOOR: u64 = 1;

/// The maximum possible value which a contract statesize PPM tax rate can be set to.
/// Set to 10,000%, meaning a contract is charged (taxed) 100 satoshis per one byte of contract state size (storage).
pub const CONTRACT_STATESIZE_PPM_TAX_RATE_CEILING: u64 = 100_000_000;

/// The default contract's shadowed liquidity PPM tax rate.
/// Initially set to none, meaning no tax is applied to contract's shadowed liquidity.
/// Subject to change via The Governance Contract.
const CONTRACT_SHADOWED_LIQUIDITY_PPM_TAX_RATE_DEFAULT: u64 = 0;

/// The maximum possible value which a contract's shadowed liquidity PPM tax rate can be set to.
/// Set to 100%, meaning a contract is charged (taxed) 1 satoshi per one satoshi of contract shadow liquidity.
pub const CONTRACT_SHADOWED_LIQUIDITY_PPM_TAX_RATE_CEILING: u64 = 1_000_000;

/// The default contract's non-shadowed liquidity PPM tax rate.
/// Initially set to none, meaning no tax is applied to contract's non-shadowed liquidity.
/// Subject to change via The Governance Contract.
const CONTRACT_NON_SHADOWED_LIQUIDITY_PPM_TAX_RATE_DEFAULT: u64 = 0;

/// The maximum possible value which a contract's non-shadowed liquidity PPM tax rate can be set to.
/// Set to 100%, meaning a contract is charged (taxed) 1 satoshi per one satoshi of contract shadow liquidity.
pub const CONTRACT_NON_SHADOWED_LIQUIDITY_PPM_TAX_RATE_CEILING: u64 = 1_000_000;

/// The default account termination check period.
/// Initially set to 4,096 batch heights, meaning an account termination check will be performed every 4,096 batch height.
/// Subject to change via The Governance Contract.
const ACCOUNT_TERMINATION_CHECK_PERIOD_DEFAULT: u64 = 4_096;

/// The minimum possible value which an account termination check period can be set to.
/// Set to 128 batch heights. The Governance Contract can never set the batch height below this value.
pub const ACCOUNT_TERMINATION_CHECK_PERIOD_FLOOR: u64 = 128;

/// The maximum possible value which an account termination check period can be set to.
/// Set to 1,000,000 batch heights. The Governance Contract can never set the batch height above this value.
pub const ACCOUNT_TERMINATION_CHECK_PERIOD_CEILING: u64 = 1_000_000;

/// The default contract termination check period.
/// Initially set to 4,096 batch heights, meaning a contract termination check will be performed every 4,096 batch height.
/// Subject to change via The Governance Contract.
const CONTRACT_TERMINATION_CHECK_PERIOD_DEFAULT: u64 = 4_096;

/// The minimum possible value which a contract termination check period can be set to.
/// Set to 128 batch heights. The Governance Contract can never set the batch height below this value.
pub const CONTRACT_TERMINATION_CHECK_PERIOD_FLOOR: u64 = 128;

/// The maximum possible value which a contract termination check period can be set to.
/// Set to 10,000,000 batch heights. The Governance Contract can never set the batch height above this value.
pub const CONTRACT_TERMINATION_CHECK_PERIOD_CEILING: u64 = 10_000_000;

/// Holder for protocol-level params.
#[derive(Clone, Debug)]
pub struct ParamsHolder {
    pub account_can_initially_deploy_liquidity: bool,
    pub account_can_initially_deploy_contract: bool,
    pub move_entry_base_fee: u64,
    pub call_entry_base_fee: u64,
    pub call_entry_ppm_calldata_bytesize_fee: u64,
    pub liftup_entry_base_fee: u64,
    pub swapout_entry_base_fee: u64,
    pub config_entry_base_fee: u64,
    pub config_entry_per_config_byte_fee: u64,
    pub deploy_entry_base_fee: u64,
    pub deploy_entry_per_program_byte_fee: u64,
    pub liftup_entry_per_lift_base_fee: u64,
    pub move_ppm_liquidity_fee: u64,
    pub in_call_ppm_liquidity_fee: u64,
    // Pleb inactivity period parameters.
    pub configured_pleb_inactivity_termination_period_seconds: u64,
    pub unconfigured_pleb_inactivity_termination_period_seconds: u64,
    // Mutable contract vitality sucked termination value.
    pub mutable_contract_vitality_sucked_termination_value: i64,
    // Contract tax rate parameters.
    pub contract_statesize_ppm_tax_rate: u64,
    pub contract_binsize_ppm_tax_rate: u64,
    pub contract_shadowed_liquidity_ppm_tax_rate: u64,
    pub contract_non_shadowed_liquidity_ppm_tax_rate: u64,
    // Termination check period parameters.
    pub account_termination_check_period: u64,
    pub contract_termination_check_period: u64,
    // Opcode ops parameters.
    pub opcode_ops: OpcodeOpsParams,
}

impl ParamsHolder {
    /// Constructs a fresh new params holder with default protocol values.
    pub fn origin_params_holder() -> Self {
        Self {
            account_can_initially_deploy_liquidity: true,
            account_can_initially_deploy_contract: true,
            move_entry_base_fee: 10,
            call_entry_base_fee: 10,
            call_entry_ppm_calldata_bytesize_fee: 1_000_000,
            liftup_entry_base_fee: 10,
            swapout_entry_base_fee: 10,
            config_entry_base_fee: 10,
            config_entry_per_config_byte_fee: 1,
            deploy_entry_base_fee: 10,
            deploy_entry_per_program_byte_fee: 10,
            liftup_entry_per_lift_base_fee: 50,
            move_ppm_liquidity_fee: 1000,
            in_call_ppm_liquidity_fee: 1000,
            // Pleb inactivity period parameters.
            configured_pleb_inactivity_termination_period_seconds:
                CONFIGURED_PLEB_INACTIVITY_TERMINATION_PERIOD_SECONDS_DEFAULT,
            unconfigured_pleb_inactivity_termination_period_seconds:
                UNCONFIGURED_PLEB_INACTIVITY_TERMINATION_PERIOD_SECONDS_DEFAULT,
            // Mutable contract vitality sucked termination value.
            mutable_contract_vitality_sucked_termination_value:
                MUTABLE_CONTRACT_VITALITY_SUCKED_TERMINATION_VALUE_DEFAULT,
            // Contract tax rate parameters.
            contract_statesize_ppm_tax_rate: CONTRACT_STATESIZE_PPM_TAX_RATE_DEFAULT,
            contract_binsize_ppm_tax_rate: CONTRACT_BINSIZE_PPM_TAX_RATE_DEFAULT,
            contract_shadowed_liquidity_ppm_tax_rate:
                CONTRACT_SHADOWED_LIQUIDITY_PPM_TAX_RATE_DEFAULT,
            contract_non_shadowed_liquidity_ppm_tax_rate:
                CONTRACT_NON_SHADOWED_LIQUIDITY_PPM_TAX_RATE_DEFAULT,
            // Termination check period parameters.
            account_termination_check_period: ACCOUNT_TERMINATION_CHECK_PERIOD_DEFAULT,
            contract_termination_check_period: CONTRACT_TERMINATION_CHECK_PERIOD_DEFAULT,
            // Opcode ops parameters.
            opcode_ops: OpcodeOpsParams::origin(),
        }
    }
}
