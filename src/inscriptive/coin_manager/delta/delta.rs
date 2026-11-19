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

/// A struct for containing ephemeral state differences to be applied for 'CoinManager'.
#[derive(Clone)]
pub struct CMDelta {
    /// ACCOUNT RELATED VALUES ///
    /// ------------------------------------------------------------
    // New accounts to register.
    pub new_accounts_to_register: HashMap<AccountKey, SatoshiAmount>,

    // Updated account balances for a given account.
    pub updated_account_balances: HashMap<AccountKey, SatoshiAmount>,

    // Updated global shadow allocs sums for a given account (sum across all contracts).
    pub updated_global_shadow_allocs_sums: HashMap<AccountKey, SatiSatoshiAmount>,

    // Accounts to erase from the coin manager.
    pub accounts_to_erase: Vec<AccountKey>,

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

    // Contracts to erase from the coin manager.
    pub contracts_to_erase: Vec<ContractId>,
}

impl CMDelta {
    /// Constructs a fresh new coin manager delta.
    pub fn fresh_new() -> Self {
        Self {
            new_accounts_to_register: HashMap::new(),
            updated_account_balances: HashMap::new(),
            updated_global_shadow_allocs_sums: HashMap::new(),
            accounts_to_erase: Vec::new(),
            new_contracts_to_register: HashMap::new(),
            allocs_list: HashMap::new(),
            deallocs_list: HashMap::new(),
            updated_contract_balances: HashMap::new(),
            updated_shadow_spaces: HashMap::new(),
            contracts_to_erase: Vec::new(),
        }
    }

    /// Clears all values.
    pub fn flush(&mut self) {
        self.new_accounts_to_register.clear();
        self.updated_account_balances.clear();
        self.updated_global_shadow_allocs_sums.clear();
        self.accounts_to_erase.clear();
        self.new_contracts_to_register.clear();
        self.allocs_list.clear();
        self.deallocs_list.clear();
        self.updated_contract_balances.clear();
        self.updated_shadow_spaces.clear();
        self.contracts_to_erase.clear();
    }

    /// Checks if an account has just been ephemerally registered in the delta.
    pub fn is_account_ephemerally_registered(&self, account_key: AccountKey) -> bool {
        self.new_accounts_to_register.contains_key(&account_key)
    }

    /// Checks if a contract has just been ephemerally registered in the delta.
    pub fn is_contract_ephemerally_registered(&self, contract_id: ContractId) -> bool {
        self.new_contracts_to_register.contains_key(&contract_id)
    }

    /// Checks if an account has just been ephemerally marked for erasure in the delta.
    pub fn is_account_ephemerally_erased(&self, account_key: AccountKey) -> bool {
        self.accounts_to_erase.contains(&account_key)
    }

    /// Checks if a contract has just been ephemerally marked for erasure in the delta.
    pub fn is_contract_ephemerally_erased(&self, contract_id: ContractId) -> bool {
        self.contracts_to_erase.contains(&contract_id)
    }

    /// ACCOUNT RELATED METHODS ///
    /// ------------------------------------------------------------
    /// Epheremally registers an account in the delta.
    pub fn ephemerally_register_account(
        &mut self,
        account_key: AccountKey,
        initial_account_balance: SatoshiAmount,
    ) {
        self.new_accounts_to_register
            .insert(account_key, initial_account_balance);
    }

    /// Epheremally updates an account's balance.
    pub fn ephemerally_update_account_balance(
        &mut self,
        account_key: AccountKey,
        balance: SatoshiAmount,
    ) {
        self.updated_account_balances.insert(account_key, balance);
    }

    /// Epheremally updates an account's global shadow allocs sum.
    pub fn ephemerally_update_account_global_shadow_allocs_sum(
        &mut self,
        account_key: AccountKey,
        global_shadow_allocs_sum: SatiSatoshiAmount,
    ) {
        self.updated_global_shadow_allocs_sums
            .insert(account_key, global_shadow_allocs_sum);
    }

    /// Epheremally marks an account for erasure in the delta.
    pub fn ephemerally_erase_account(&mut self, account_key: AccountKey) {
        self.accounts_to_erase.push(account_key);
    }

    /// CONTRACT RELATED METHODS ///
    /// ------------------------------------------------------------
    /// Epheremally registers a contract in the delta.
    pub fn ephemerally_register_contract(
        &mut self,
        contract_id: ContractId,
        initial_contract_balance: SatoshiAmount,
    ) {
        self.new_contracts_to_register
            .insert(contract_id, initial_contract_balance);
    }

    /// Epheremally updates a contract's balance.
    pub fn ephemerally_update_contract_balance(
        &mut self,
        contract_id: ContractId,
        balance: SatoshiAmount,
    ) {
        self.updated_contract_balances.insert(contract_id, balance);
    }

    /// Epheremally marks a contract for erasure in the delta.
    pub fn ephemerally_erase_contract(&mut self, contract_id: ContractId) {
        self.contracts_to_erase.push(contract_id);
    }

    /// Epheremally inserts an allocation record to the allocs list.
    pub fn ephemerally_insert_alloc(&mut self, contract_id: ContractId, account_key: AccountKey) {
        self.allocs_list
            .entry(contract_id)
            .or_default()
            .push(account_key);
    }

    /// Epheremally inserts a deallocation record to the deallocs list.
    pub fn ephemerally_insert_dealloc(&mut self, contract_id: ContractId, account_key: AccountKey) {
        self.deallocs_list
            .entry(contract_id)
            .or_default()
            .push(account_key);
    }

    /// Returns the list of accounts whose coin balances or allocations are changed in one way or another.
    pub fn coingap_accounts_list(&self) -> Vec<AccountKey> {
        // 1 Initialize the affected accounts list.
        let mut affected_accounts: Vec<AccountKey> = Vec::new();

        // 2 Add the newly registered accounts with non-zero initial balances.
        for (account_key, initial_balance) in self.new_accounts_to_register.iter() {
            // 2.1 Check if the initial balance is non-zero.
            if *initial_balance != 0 {
                // 2.1.1 Insert if not already present.
                if !affected_accounts.contains(account_key) {
                    affected_accounts.push(account_key.to_owned());
                }
            }
        }

        // 3 Add the accounts that have their balances updated.
        for (account_key, _) in self.updated_account_balances.iter() {
            // 3.1 Insert if not already present.
            if !affected_accounts.contains(account_key) {
                affected_accounts.push(account_key.to_owned());
            }
        }

        // 4 Now do for global shadow allocs sums.
        for (account_key, _) in self.updated_global_shadow_allocs_sums.iter() {
            // 4.1 Insert if not already present.
            if !affected_accounts.contains(account_key) {
                affected_accounts.push(account_key.to_owned());
            }
        }

        // 5 Return the affected accounts list.
        affected_accounts
    }
}
