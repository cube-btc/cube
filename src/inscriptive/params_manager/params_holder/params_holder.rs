/// Holder for protocol-level params.
#[derive(Clone)]
pub struct ParamsHolder {
    pub account_can_initially_deploy_liquidity: bool,
    pub account_can_initially_deploy_contract: bool,
    pub move_entry_base_fee: u64,
    pub call_entry_base_fee: u64,
    pub call_entry_ppm_calldata_bytesize_fee: u64,
    pub liftup_entry_base_fee: u64,
    pub liftup_entry_per_lift_base_fee: u64,
    pub move_ppm_liquidity_fee: u64,
    pub in_call_ppm_liquidity_fee: u64,
}

impl ParamsHolder {
    /// Constructs a fresh new params holder with default protocol values.
    pub fn fresh_new() -> Self {
        Self {
            account_can_initially_deploy_liquidity: true,
            account_can_initially_deploy_contract: true,
            move_entry_base_fee: 10,
            call_entry_base_fee: 10,
            call_entry_ppm_calldata_bytesize_fee: 1_000_000,
            liftup_entry_base_fee: 10,
            liftup_entry_per_lift_base_fee: 50,
            move_ppm_liquidity_fee: 1000,
            in_call_ppm_liquidity_fee: 1000,
        }
    }
}
