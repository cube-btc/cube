use crate::executive::executable::executable::Executable;
use std::collections::HashMap;

/// secp256k1 public key of an account.
type AccountKey = [u8; 32];

/// BLS key of an account.
type AccountBLSKey = [u8; 48];

/// Secondary aggregation key of an account (in case needed for post-quantum security).
type AccountSecondaryAggregationKey = Vec<u8>;

/// Contract ID.
type ContractId = [u8; 32];

/// Epheremal call counter gap to be applied to an account or contract.
type CallCounterDelta = u16;

/// A struct for containing epheremal state differences to be applied for 'RegisteryManager'.
#[derive(Clone)]
pub struct RMDelta {
    // ACCOUNT RELATED VALUES ///
    /// ------------------------------------------------------------
    // New accounts to register.
    pub new_accounts_to_register: Vec<(
        AccountKey,
        Option<AccountBLSKey>,
        Option<AccountSecondaryAggregationKey>,
    )>,

    // Updated account call counters for a given account.
    pub updated_account_call_counters: HashMap<AccountKey, CallCounterDelta>,

    // Updated primary BLS keys for a given account.
    pub updated_bls_keys: HashMap<AccountKey, AccountBLSKey>,

    // Updated secondary aggregation keys for a given account.
    pub updated_secondary_aggregation_keys: HashMap<AccountKey, AccountSecondaryAggregationKey>,

    // CONTRACT RELATED VALUES ///
    /// ------------------------------------------------------------
    // New contracts to register.
    pub new_contracts_to_register: Vec<(ContractId, Executable)>,

    // Updated contract call counters for a given contract.
    pub updated_contract_call_counters: HashMap<ContractId, CallCounterDelta>,
}

impl RMDelta {
    /// Constructs a fresh new registery manager delta.
    pub fn fresh_new() -> Self {
        Self {
            new_accounts_to_register: Vec::new(),
            updated_account_call_counters: HashMap::new(),
            updated_bls_keys: HashMap::new(),
            updated_secondary_aggregation_keys: HashMap::new(),
            new_contracts_to_register: Vec::new(),
            updated_contract_call_counters: HashMap::new(),
        }
    }

    /// Clears all values.
    pub fn flush(&mut self) {
        self.new_accounts_to_register.clear();
        self.updated_account_call_counters.clear();
        self.updated_bls_keys.clear();
        self.updated_secondary_aggregation_keys.clear();
        self.new_contracts_to_register.clear();
        self.updated_contract_call_counters.clear();
    }

    /// Checks if an account has just been epheremally registered in the delta.
    pub fn is_account_epheremally_registered(&self, account_key: AccountKey) -> bool {
        self.new_accounts_to_register
            .iter()
            .any(|(key, _, _)| key == &account_key)
    }

    /// Checks if a contract has just been epheremally registered in the delta.
    pub fn is_contract_epheremally_registered(&self, contract_id: ContractId) -> bool {
        self.new_contracts_to_register
            .iter()
            .any(|(id, _)| id == &contract_id)
    }

    /// Epheremally registers an account in the delta.
    pub fn epheremally_register_account(
        &mut self,
        account_key: AccountKey,
        primary_bls_key: Option<AccountBLSKey>,
        secondary_aggregation_key: Option<AccountSecondaryAggregationKey>,
    ) {
        self.new_accounts_to_register.push((
            account_key,
            primary_bls_key,
            secondary_aggregation_key,
        ));
    }

    /// Epheremally registers a contract in the delta.
    pub fn epheremally_register_contract(
        &mut self,
        contract_id: ContractId,
        executable: Executable,
    ) {
        self.new_contracts_to_register
            .push((contract_id, executable));
    }

    /// Epheremally increments the call counter delta of an account by one.
    pub fn epheremally_increment_account_call_counter_delta_by_one(
        &mut self,
        account_key: AccountKey,
    ) {
        // 1 Check if the call counter delta exists in the delta.
        match self.updated_account_call_counters.get(&account_key) {
            // 1.a The call counter delta exists in the delta.
            Some(call_counter_delta) => {
                // 1.a.1 Increment the call counter delta by one.
                let new_call_counter_delta = call_counter_delta + 1;

                // 1.a.2 Epheremally update the call counter delta in the delta.
                self.updated_account_call_counters
                    .insert(account_key, new_call_counter_delta);
            }
            // 1.b The call counter delta does not exist in the delta.
            None => {
                // 1.b.1 Call counter delta starts from one if it does not exist in the delta.
                self.updated_account_call_counters.insert(account_key, 1);
            }
        }
    }

    /// Epheremally increments the call counter delta of a contract by one.
    pub fn epheremally_increment_contract_call_counter_delta_by_one(
        &mut self,
        contract_id: ContractId,
    ) {
        // 1 Check if the call counter delta exists in the delta.
        match self.updated_contract_call_counters.get(&contract_id) {
            // 1.a The call counter delta exists in the delta.
            Some(call_counter_delta) => {
                // 1.a.1 Increment the call counter delta by one.
                let new_call_counter_delta = call_counter_delta + 1;

                // 1.a.2 Epheremally update the call counter delta in the delta.
                self.updated_contract_call_counters
                    .insert(contract_id, new_call_counter_delta);
            }
            // 1.b The call counter delta does not exist in the delta.
            None => {
                // 1.b.1 Call counter delta starts from one if it does not exist in the delta.
                self.updated_contract_call_counters.insert(contract_id, 1);
            }
        }
    }

    /// Epheremally sets an account's BLS key.
    pub fn epheremally_set_account_bls_key(
        &mut self,
        account_key: AccountKey,
        bls_key: AccountBLSKey,
    ) -> Option<AccountBLSKey> {
        self.updated_bls_keys.insert(account_key, bls_key)
    }

    /// Epheremally updates an account's secondary aggregation key.
    pub fn epheremally_update_account_secondary_aggregation_key(
        &mut self,
        account_key: AccountKey,
        secondary_aggregation_key: AccountSecondaryAggregationKey,
    ) {
        self.updated_secondary_aggregation_keys
            .insert(account_key, secondary_aggregation_key);
    }
}
