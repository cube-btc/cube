use crate::inscriptive::coin_manager::bodies::contract_body::shadow_space::shadow_space::ShadowSpace;
use std::collections::HashMap;

/// Account key.
#[allow(non_camel_case_types)]
type AccountKey = [u8; 32];

/// Contract ID.
#[allow(non_camel_case_types)]
type ContractId = [u8; 32];

/// Satoshi amount.
#[allow(non_camel_case_types)]
type SatoshiAmount = u64;

/// Sati-satoshi amount.
#[allow(non_camel_case_types)]
type SatiSatoshiAmount = u128;

/// A struct for containing epheremal state differences to be applied for 'CoinManager'.
#[derive(Clone)]
pub struct CMDelta {
    /// ACCOUNT RELATED VALUES ///
    /// ------------------------------------------------------------
    // New accounts to register.
    pub new_accounts_to_register: HashMap<AccountKey, SatoshiAmount>,

    // Updated account balances for a given account.
    pub updated_account_balances: HashMap<AccountKey, SatoshiAmount>,

    // Updated shadow allocs sums for a given account.
    pub updated_shadow_allocs_sums: HashMap<AccountKey, SatiSatoshiAmount>,

    /// CONTRACT RELATED VALUES ///
    /// ------------------------------------------------------------
    // New contracts to register.
    pub new_contracts_to_register: HashMap<ContractId, SatoshiAmount>,

    // New accounts to allocate for a given contract.
    pub allocs_list: HashMap<ContractId, Vec<AccountKey>>,

    // Existing accounts to deallocate for a given contract.
    pub deallocs_list: HashMap<ContractId, Vec<AccountKey>>,

    // Updated contract balances for a given contract.
    pub updated_contract_balances: HashMap<ContractId, SatoshiAmount>,

    // Updated shadow spaces for a given contract.
    pub updated_shadow_spaces: HashMap<ContractId, ShadowSpace>,
}

impl CMDelta {
    /// Constructs a fresh new coin manager delta.
    pub fn fresh_new() -> Self {
        Self {
            new_accounts_to_register: HashMap::new(),
            updated_account_balances: HashMap::new(),
            updated_shadow_allocs_sums: HashMap::new(),
            new_contracts_to_register: HashMap::new(),
            allocs_list: HashMap::new(),
            deallocs_list: HashMap::new(),
            updated_contract_balances: HashMap::new(),
            updated_shadow_spaces: HashMap::new(),
        }
    }

    /// Clears all values.
    pub fn flush(&mut self) {
        self.new_accounts_to_register.clear();
        self.updated_account_balances.clear();
        self.updated_shadow_allocs_sums.clear();
        self.new_contracts_to_register.clear();
        self.allocs_list.clear();
        self.deallocs_list.clear();
        self.updated_contract_balances.clear();
        self.updated_shadow_spaces.clear();
    }

    /// ACCOUNT RELATED METHODS ///
    /// ------------------------------------------------------------

    /// Epheremally updates an account's balance.
    pub fn epheremally_update_account_balance(
        &mut self,
        account_key: AccountKey,
        balance: SatoshiAmount,
    ) {
        self.updated_account_balances.insert(account_key, balance);
    }

    /// Epheremally updates an account's shadow allocs sum.
    pub fn epheremally_update_account_shadow_allocs_sum(
        &mut self,
        account_key: AccountKey,
        shadow_allocs_sum: SatiSatoshiAmount,
    ) {
        self.updated_shadow_allocs_sums
            .insert(account_key, shadow_allocs_sum);
    }

    /// CONTRACT RELATED METHODS ///
    /// ------------------------------------------------------------

    /// Epheremally updates a contract's balance.
    pub fn epheremally_update_contract_balance(
        &mut self,
        contract_id: ContractId,
        balance: SatoshiAmount,
    ) {
        self.updated_contract_balances.insert(contract_id, balance);
    }

    /// Epheremally inserts an allocation record to the allocs list.
    pub fn epheremally_insert_alloc(&mut self, contract_id: ContractId, account_key: AccountKey) {
        self.allocs_list
            .entry(contract_id)
            .or_insert_with(Vec::new)
            .push(account_key);
    }

    /// Epheremally inserts a deallocation record to the deallocs list.
    pub fn epheremally_insert_dealloc(&mut self, contract_id: ContractId, account_key: AccountKey) {
        self.deallocs_list
            .entry(contract_id)
            .or_insert_with(Vec::new)
            .push(account_key);
    }

    /// Returns the list of accounts that are affected by the `CoinManager`.
    pub fn affected_accounts_list(&self) -> Vec<AccountKey> {
        // 1 Initialize the affected accounts list.
        let mut affected_accounts: Vec<AccountKey> = Vec::new();

        // 2 Add the accounts that have their balances updated.
        for (account_key, _) in self.updated_account_balances.iter() {
            // 2.1 Insert if not already present.
            if !affected_accounts.contains(account_key) {
                affected_accounts.push(account_key.to_owned());
            }
        }

        // 3 Now do for shadow allocs sums.
        for (account_key, _) in self.updated_shadow_allocs_sums.iter() {
            // 3.1 Insert if not already present.
            if !affected_accounts.contains(account_key) {
                affected_accounts.push(account_key.to_owned());
            }
        }

        // 4 Return the affected accounts list.
        affected_accounts
    }
}
