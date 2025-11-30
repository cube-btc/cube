use crate::inscriptive::registery_manager::bodies::account_body::flame_config::flame_config::FlameConfig;
use std::collections::HashMap;

/// Account Key.
type AccountKey = [u8; 32];

/// Contract ID.
type ContractId = [u8; 32];

/// Epheremal call counter gap to be applied to an account or contract.
type CallCounterDelta = u16;

/// Secondary aggregation key of an account (in case needed for post-quantum security).
type SecondaryAggregationKey = Vec<u8>;

/// A struct for containing epheremal state differences to be applied for 'RegisteryManager'.
pub struct RMDelta {
    // ACCOUNT RELATED VALUES ///
    /// ------------------------------------------------------------
    // New accounts to register.
    pub new_accounts_to_register: Vec<AccountKey>,

    // Updated account call counters for a given account.
    pub updated_account_call_counters: HashMap<AccountKey, CallCounterDelta>,

    // Updated secondary aggregation keys for a given account.
    pub updated_secondary_aggregation_keys: HashMap<AccountKey, SecondaryAggregationKey>,

    // Updated flame configs for a given account.
    pub updated_flame_configs: HashMap<AccountKey, FlameConfig>,

    // CONTRACT RELATED VALUES ///
    /// ------------------------------------------------------------
    // New contracts to register.
    pub new_contracts_to_register: Vec<ContractId>,

    // Updated contract call counters for a given contract.
    pub updated_contract_call_counters: HashMap<ContractId, CallCounterDelta>,
}

impl RMDelta {
    /// Constructs a fresh new registery manager delta.
    pub fn fresh_new() -> Self {
        Self {
            new_accounts_to_register: Vec::new(),
            updated_account_call_counters: HashMap::new(),
            updated_secondary_aggregation_keys: HashMap::new(),
            updated_flame_configs: HashMap::new(),
            new_contracts_to_register: Vec::new(),
            updated_contract_call_counters: HashMap::new(),
        }
    }

    /// Clears all values.
    pub fn flush(&mut self) {
        self.new_accounts_to_register.clear();
        self.updated_account_call_counters.clear();
        self.updated_secondary_aggregation_keys.clear();
        self.updated_flame_configs.clear();
        self.new_contracts_to_register.clear();
        self.updated_contract_call_counters.clear();
    }
}
