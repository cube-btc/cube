use crate::inscriptive::coin_holder::bodies::account_body::account_body::CHAccountBody;
use crate::inscriptive::coin_holder::bodies::contract_body::contract_body::CHContractBody;
use crate::inscriptive::coin_holder::bodies::contract_body::shadow_space::shadow_space::ShadowSpace;
use crate::inscriptive::coin_holder::deltas::account_delta::account_delta::CHAccountDelta;
use crate::inscriptive::coin_holder::deltas::contract_delta::contract_delta::CHContractDelta;
use crate::inscriptive::coin_holder::errors::apply_changes_errors::{
    CHAccountApplyChangesError, CHApplyChangesError, CHContractApplyChangesError,
};
use crate::inscriptive::coin_holder::errors::balance_update_errors::{
    CHAccountBalanceDownError, CHAccountBalanceUpError, CHContractBalanceDownError,
    CHContractBalanceUpError,
};
use crate::inscriptive::coin_holder::errors::construction_errors::{
    CHConstructionAccountError, CHConstructionContractError, CHConstructionError,
};
use crate::inscriptive::coin_holder::errors::register_errors::{
    CHRegisterAccountError, CHRegisterContractError,
};
use crate::inscriptive::coin_holder::errors::shadow_alloc_errors::{
    CHContractShadowAllocAccountError, CHContractShadowDeallocAccountError,
};
use crate::inscriptive::coin_holder::errors::shadow_update_errors::{
    CHAccountShadowAllocsSumDownError, CHAccountShadowAllocsSumUpError, CHShadowDownAllError,
    CHShadowDownError, CHShadowUpAllError, CHShadowUpError,
};
use crate::operative::Chain;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Account key.
#[allow(non_camel_case_types)]
type ACCOUNT_KEY = [u8; 32];

/// Contract ID.
#[allow(non_camel_case_types)]
type CONTRACT_ID = [u8; 32];

/// Sati-satoshi amount.
#[allow(non_camel_case_types)]
type SATI_SATOSHI_AMOUNT = u128;

/// Special db key for the account balance (0x00..).
const ACCOUNT_BALANCE_SPECIAL_KEY: [u8; 1] = [0x00; 1];
/// Special db key for the account shadow allocs sum (0x01..).
const ACCOUNT_ALLOCS_SUM_SPECIAL_KEY: [u8; 1] = [0x01; 1];

/// Special db key for the contract balance (0x00..).
const CONTRACT_BALANCE_SPECIAL_KEY: [u8; 32] = [0x00; 32];
/// Special db key for the allocs sum (0x01..).
const CONTRACT_ALLOCS_SUM_SPECIAL_KEY: [u8; 32] = [0x01; 32];

/// A database manager for handling account and contract balances & shadow space allocations.
pub struct CoinHolder {
    // IN-MEMORY STATES
    in_memory_accounts: HashMap<ACCOUNT_KEY, CHAccountBody>,
    in_memory_contracts: HashMap<CONTRACT_ID, CHContractBody>,

    // ON-DISK STATES
    on_disk_accounts: sled::Db,
    on_disk_contracts: sled::Db,

    // STATE DIFFERENCES TO BE APPLIED
    delta_accounts: CHAccountDelta,
    delta_contracts: CHContractDelta,

    // BACKUP OF STATE DIFFERENCES IN CASE OF ROLLBACK
    backup_of_delta_accounts: CHAccountDelta,
    backup_of_delta_contracts: CHContractDelta,
}

/// Guarded coin holder.
#[allow(non_camel_case_types)]
pub type COIN_HOLDER = Arc<Mutex<CoinHolder>>;

impl CoinHolder {
    pub fn new(chain: Chain) -> Result<COIN_HOLDER, CHConstructionError> {
        // 1. Open the account db.
        let account_db_path = format!("db/{}/coin/account", chain.to_string());
        let account_db = sled::open(account_db_path).map_err(|e| {
            CHConstructionError::AccountConstructionError(CHConstructionAccountError::DBOpenError(
                e,
            ))
        })?;

        // 2. Open the contract db.
        let contract_db_path = format!("db/{}/coin/contract", chain.to_string());
        let contract_db = sled::open(contract_db_path).map_err(|e| {
            CHConstructionError::ContractConstructionError(
                CHConstructionContractError::DBOpenError(e),
            )
        })?;

        // 3. Initialize the in-memory lists of account and contract bodies.
        let mut account_bodies = HashMap::<ACCOUNT_KEY, CHAccountBody>::new();
        let mut contract_bodies = HashMap::<CONTRACT_ID, CHContractBody>::new();

        // 4. Collect account bodies from the account database.
        for tree_name in account_db.tree_names() {
            // Deserialize account key bytes from tree name.
            let account_key: [u8; 32] = tree_name.as_ref().try_into().map_err(|_| {
                CHConstructionError::AccountConstructionError(
                    CHConstructionAccountError::UnableToDeserializeAccountKeyBytesFromTreeName(
                        tree_name.to_vec(),
                    ),
                )
            })?;

            // Open the tree.
            let tree = account_db.open_tree(tree_name).map_err(|e| {
                CHConstructionError::AccountConstructionError(
                    CHConstructionAccountError::TreeOpenError(account_key, e),
                )
            })?;

            let mut account_balance: u64 = 0;
            let mut account_shadow_allocs_sum: u128 = 0;

            // iterate over all items in the tree.
            for (index, item) in tree.iter().enumerate() {
                // Get the key and value.
                let (key, value) = match item {
                    Ok((k, v)) => (k, v),
                    Err(e) => {
                        return Err(CHConstructionError::AccountConstructionError(
                            CHConstructionAccountError::TreeIterError(index, e),
                        ));
                    }
                };

                // Deserialize the key bytes.
                let tree_key_byte: [u8; 1] = key.as_ref().try_into().map_err(|_| {
                    CHConstructionError::AccountConstructionError(
                        CHConstructionAccountError::UnableToDeserializeKeyBytesFromTreeKey(
                            account_key,
                            index,
                            key.to_vec(),
                        ),
                    )
                })?;

                // Match the tree key bytes.
                match tree_key_byte {
                    // If the key is (0x00..), it is a special key that corresponds to the account balance value.
                    ACCOUNT_BALANCE_SPECIAL_KEY => {
                        // Deserialize the value bytes.
                        let account_balance_deserialized: u64 =
                            u64::from_le_bytes(value.as_ref().try_into().map_err(|_| {
                                CHConstructionError::AccountConstructionError(CHConstructionAccountError::UnableToDeserializeAccountBalanceFromTreeValue(
                                    account_key,
                                    index,
                                    tree_key_byte,
                                    value.to_vec(),
                                ),
                                )
                            })?);

                        // Update the account balance.
                        account_balance = account_balance_deserialized;
                    }
                    // If the key is (0x01..), it is a special key that corresponds to the account shadow allocs sum value.
                    ACCOUNT_ALLOCS_SUM_SPECIAL_KEY => {
                        // Deserialize the value bytes.
                        let account_shadow_allocs_sum_deserialized: u128 =
                            u128::from_le_bytes(value.as_ref().try_into().map_err(|_| {
                                CHConstructionError::AccountConstructionError(CHConstructionAccountError::UnableToDeserializeAccountShadowAllocsSumFromTreeValue(
                                    account_key,
                                    index,
                                    tree_key_byte,
                                    value.to_vec(),
                                ))
                            })?);

                        // Update the account shadow allocs sum.
                        account_shadow_allocs_sum = account_shadow_allocs_sum_deserialized;
                    }
                    _ => {
                        // This key is a normal account key that corresponds to an account allocation.
                        return Err(CHConstructionError::AccountConstructionError(
                            CHConstructionAccountError::InvalidTreeKeyEncountered(
                                account_key,
                                tree_key_byte.to_vec(),
                            ),
                        ));
                    }
                }

                // Construct the account body.
                let account_body = CHAccountBody {
                    balance: account_balance,
                    shadow_allocs_sum: account_shadow_allocs_sum,
                };

                // Insert the account body into the account bodies list.
                account_bodies.insert(account_key, account_body);
            }
        }

        // 5. Collect contract bodies from the contract database.
        for tree_name in contract_db.tree_names() {
            // Deserialize contract id bytes from tree name.
            let contract_id: [u8; 32] = tree_name.as_ref().try_into().map_err(|_| {
                CHConstructionError::ContractConstructionError(
                    CHConstructionContractError::UnableToDeserializeContractIDBytesFromTreeName(
                        tree_name.to_vec(),
                    ),
                )
            })?;

            // Open the tree.
            let tree = contract_db.open_tree(&tree_name).map_err(|e| {
                CHConstructionError::ContractConstructionError(
                    CHConstructionContractError::TreeOpenError(contract_id, e),
                )
            })?;

            // Initialize the list of shadow space allocations.
            let mut allocs = HashMap::<ACCOUNT_KEY, SATI_SATOSHI_AMOUNT>::new();

            // Initialize the allocs sum to zero.
            let mut allocs_sum: u64 = 0;

            // Initialize the contract balance to zero.
            let mut contract_balance: u64 = 0;

            // Iterate over all items in the tree.
            for (index, item) in tree.iter().enumerate() {
                // Get the key and value.
                let (key, value) = match item {
                    Ok((k, v)) => (k, v),
                    Err(e) => {
                        return Err(CHConstructionError::ContractConstructionError(
                            CHConstructionContractError::TreeIterError(contract_id, index, e),
                        ));
                    }
                };

                // Deserialize the key bytes.
                let tree_key_bytes: [u8; 32] = key.as_ref().try_into().map_err(|_| {
                    CHConstructionError::ContractConstructionError(
                        CHConstructionContractError::UnableToDeserializeKeyBytesFromTreeKey(
                            contract_id,
                            index,
                            key.to_vec(),
                        ),
                    )
                })?;

                // Match the tree key bytes.
                match tree_key_bytes {
                    // If the key is (0x00..), it is a special key that corresponds to the contract balance value.
                    CONTRACT_BALANCE_SPECIAL_KEY => {
                        // Deserialize the value bytes.
                        let contract_balance_value_in_satoshis: u64 =
                                u64::from_le_bytes(value.as_ref().try_into().map_err(|_| {
                                    CHConstructionError::ContractConstructionError(CHConstructionContractError::UnableToDeserializeContractBalanceFromTreeValue(
                                        contract_id,
                                        index,
                                        tree_key_bytes,
                                        value.to_vec(),
                                    ))
                                })?);

                        // Update the contract balance.
                        contract_balance = contract_balance_value_in_satoshis;
                    }
                    // If the key is (0x01..), it is a special key that corresponds to the allocs sum value.
                    CONTRACT_ALLOCS_SUM_SPECIAL_KEY => {
                        // Deserialize the value bytes.
                        let allocs_sum_value_in_satoshis: u64 =
                                u64::from_le_bytes(value.as_ref().try_into().map_err(|_| {
                                    CHConstructionError::ContractConstructionError(CHConstructionContractError::UnableToDeserializeAllocsSumFromTreeValue(
                                        contract_id,
                                        index,
                                        tree_key_bytes,
                                        value.to_vec(),
                                    ))
                                })?);

                        // Update the shadow space allocations sum.
                        allocs_sum = allocs_sum_value_in_satoshis;
                    }
                    _ => {
                        // This key is a normal account key that corresponds to an account allocation.

                        // Deserialize the value bytes.
                        let alloc_value_in_sati_satoshis: u128 =
                                u128::from_le_bytes(value.as_ref().try_into().map_err(|_| {
                                    CHConstructionError::ContractConstructionError(CHConstructionContractError::UnableToDeserializeAllocValueFromTreeValue(
                                        contract_id,
                                        index,
                                        tree_key_bytes,
                                        value.to_vec(),
                                    ))
                                })?);

                        // Update the shadow space allocations.
                        allocs.insert(tree_key_bytes, alloc_value_in_sati_satoshis);
                    }
                }
            }

            // Check if the shadow space allocations sum exceeds the contract balance.
            if allocs_sum > contract_balance {
                return Err(CHConstructionError::ContractConstructionError(
                    CHConstructionContractError::AllocsSumExceedsTheContractBalance(
                        contract_id,
                        allocs_sum,
                        contract_balance,
                    ),
                ));
            }

            // Construct the shadow space.
            let shadow_space = ShadowSpace { allocs_sum, allocs };

            // Construct the contract body.
            let contract_body = CHContractBody {
                balance: contract_balance,
                shadow_space,
            };

            // Insert the contract body into the contract bodies list.
            contract_bodies.insert(contract_id, contract_body);
        }

        // 6. Construct the coin holder.
        let coin_holder = CoinHolder {
            in_memory_accounts: account_bodies,
            in_memory_contracts: contract_bodies,
            on_disk_accounts: account_db,
            on_disk_contracts: contract_db,
            delta_accounts: CHAccountDelta::new(),
            delta_contracts: CHContractDelta::new(),
            backup_of_delta_accounts: CHAccountDelta::new(),
            backup_of_delta_contracts: CHContractDelta::new(),
        };

        // 7. Guard the coin holder.
        let guarded_coin_holder = Arc::new(Mutex::new(coin_holder));

        // 8. Return the guarded coin holder.
        Ok(guarded_coin_holder)
    }

    /// Clones ephemeral states into the backup.
    fn backup_delta(&mut self) {
        self.backup_of_delta_accounts = self.delta_accounts.clone();
        self.backup_of_delta_contracts = self.delta_contracts.clone();
    }

    /// Restores ephemeral states from the backup.
    fn restore_delta(&mut self) {
        self.delta_accounts = self.backup_of_delta_accounts.clone();
        self.delta_contracts = self.backup_of_delta_contracts.clone();
    }

    /// Prepares the state holder prior to each execution.
    ///
    /// NOTE: Used by the Engine coordinator.
    pub fn pre_execution(&mut self) {
        // Backup the delta.
        self.backup_delta();
    }

    /// Checks if an account is registered.
    pub fn is_account_registered(&self, account_key: ACCOUNT_KEY) -> bool {
        self.in_memory_accounts.contains_key(&account_key)
    }

    /// Checks if a contract is registered.
    pub fn is_contract_registered(&self, contract_id: CONTRACT_ID) -> bool {
        self.in_memory_contracts.contains_key(&contract_id)
    }

    /// Returns the account balance for an account key in satoshis.
    pub fn get_account_balance(&self, account_key: ACCOUNT_KEY) -> Option<u64> {
        // Try to get from the delta first.
        if let Some(value) = self
            .delta_accounts
            .updated_account_balances
            .get(&account_key)
        {
            return Some(value.clone());
        }

        // And then try to get from the permanent in-memory states.
        self.in_memory_accounts
            .get(&account_key)
            .cloned()
            .map(|account_body| account_body.balance)
    }

    /// Returns the contract balance for a contract ID in satoshis.
    pub fn get_contract_balance(&self, contract_id: CONTRACT_ID) -> Option<u64> {
        // Try to get from the delta first.
        if let Some(value) = self
            .delta_contracts
            .updated_contract_balances
            .get(&contract_id)
        {
            return Some(value.clone());
        }

        // And then try to get from the permanent in-memory states.
        self.in_memory_contracts
            .get(&contract_id)
            .map(|contract_body| contract_body.balance)
    }

    /// Returns the account shadow allocs sum for an account key in sati-satoshis.
    pub fn get_account_shadow_allocs_sum_of_all_contracts_in_sati_satoshis(
        &self,
        account_key: ACCOUNT_KEY,
    ) -> Option<u128> {
        // Try to get from the delta first.
        if let Some(value) = self
            .delta_accounts
            .updated_shadow_allocs_sums
            .get(&account_key)
        {
            return Some(value.clone());
        }

        // And then try to get from the permanent in-memory states.
        self.in_memory_accounts
            .get(&account_key)
            .map(|account_body| account_body.shadow_allocs_sum)
    }

    /// Returns the account shadow allocs sum for an account key in satoshis.
    pub fn get_account_shadow_allocs_sum_of_all_contracts_in_satoshis(
        &self,
        account_key: ACCOUNT_KEY,
    ) -> Option<u64> {
        // Get the sati-satoshi value.
        let sati_satoshi_value =
            self.get_account_shadow_allocs_sum_of_all_contracts_in_sati_satoshis(account_key)?;

        // Divide by 100_000_000 to get the satoshi value.
        let satoshi_value = sati_satoshi_value / 100_000_000;

        // Return the result.
        Some(satoshi_value as u64)
    }

    /// Get the contract shadow allocation for a contract ID in satoshis.
    pub fn get_contract_allocs_sum_in_satoshis(&self, contract_id: [u8; 32]) -> Option<u64> {
        // Try to read from the delta first.
        if let Some(allocs_sum) = self.delta_contracts.updated_shadow_spaces.get(&contract_id) {
            return Some(allocs_sum.allocs_sum);
        }

        // And then try to get from the in-memory states.
        self.in_memory_contracts
            .get(&contract_id)
            .map(|body| body.shadow_space.allocs_sum)
    }

    /// Get the number of total shadow allocations of the contract.
    pub fn get_contract_num_allocs(&self, contract_id: [u8; 32]) -> Option<u64> {
        // Try to get from the delta first.
        if let Some(shadow_space) = self.delta_contracts.updated_shadow_spaces.get(&contract_id) {
            return Some(shadow_space.allocs.len() as u64);
        }

        // And then try to get from the in-memory states.
        self.in_memory_contracts
            .get(&contract_id)
            .map(|body| body.shadow_space.allocs.len() as u64)
    }

    /// Get the shadow allocation value of an account for a specific contract ID.
    pub fn get_account_shadow_alloc_value_of_a_contract_in_sati_satoshis(
        &self,
        contract_id: [u8; 32],
        account_key: ACCOUNT_KEY,
    ) -> Option<u128> {
        // Check if the account is epheremally deallocated in the delta.
        if let Some(dealloc_list) = self.delta_contracts.deallocs_list.get(&contract_id) {
            if dealloc_list.contains(&account_key) {
                return None;
            }
        }

        // Try to read from the delta first.
        if let Some(shadow_space) = self.delta_contracts.updated_shadow_spaces.get(&contract_id) {
            return shadow_space.allocs.get(&account_key).cloned();
        }

        // And then try to read from the permanent states.
        self.in_memory_contracts
            .get(&contract_id)
            .and_then(|body| body.shadow_space.allocs.get(&account_key).cloned())
    }

    /// Get the shadow allocation value of an account for a specific contract ID in satoshis.
    pub fn get_account_shadow_alloc_value_of_a_contract_in_satoshis(
        &self,
        contract_id: [u8; 32],
        account_key: ACCOUNT_KEY,
    ) -> Option<u64> {
        // Get the sati-satoshi value.
        let sati_satoshi_value = self
            .get_account_shadow_alloc_value_of_a_contract_in_sati_satoshis(
                contract_id,
                account_key,
            )?;

        // Divide by 100_000_000 to get the satoshi value.
        let satoshi_value = sati_satoshi_value / 100_000_000;

        // Return the result.
        Some(satoshi_value as u64)
    }

    /// Registers an account.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn register_account(
        &mut self,
        account_key: ACCOUNT_KEY,
    ) -> Result<(), CHRegisterAccountError> {
        // Check if the account has just been epheremally registered in the delta.
        if self
            .delta_accounts
            .new_accounts_to_register
            .contains(&account_key)
        {
            return Err(
                CHRegisterAccountError::AccountHasJustBeenEphemerallyRegistered(account_key),
            );
        }

        // Check if the account is already permanently registered.
        if self.is_account_registered(account_key) {
            return Err(CHRegisterAccountError::AccountIsAlreadyPermanentlyRegistered(account_key));
        }

        // Insert into the new accounts to register list in the delta.
        self.delta_accounts
            .new_accounts_to_register
            .push(account_key);

        // Return the result.
        Ok(())
    }

    /// Registers a contract.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn register_contract(
        &mut self,
        contract_id: [u8; 32],
    ) -> Result<(), CHRegisterContractError> {
        // Check if the contract has just been epheremally registered in the delta.
        if self
            .delta_contracts
            .new_contracts_to_register
            .contains(&contract_id)
        {
            return Err(
                CHRegisterContractError::ContractHasJustBeenEphemerallyRegistered(contract_id),
            );
        }

        // Check if the contract is already permanently registered.
        if self.is_contract_registered(contract_id) {
            return Err(
                CHRegisterContractError::ContractIsAlreadyPermanentlyRegistered(contract_id),
            );
        }

        // Insert into the new contracts to register list in the delta.
        self.delta_contracts
            .new_contracts_to_register
            .push(contract_id);

        // Return the result.
        Ok(())
    }

    /// Increases the account's balance.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn account_balance_up(
        &mut self,
        account_key: ACCOUNT_KEY,
        up_value_in_satoshis: u64,
    ) -> Result<(), CHAccountBalanceUpError> {
        // Get the old ephemeral account balance before any mutable borrows.
        let existing_account_balance_in_satoshis: u64 =
            self.get_account_balance(account_key).ok_or(
                CHAccountBalanceUpError::UnableToGetAccountBalance(account_key),
            )?;

        // Calculate the new ephemeral account balance.
        let new_account_balance_in_satoshis: u64 =
            existing_account_balance_in_satoshis + up_value_in_satoshis;

        // Retrieve the mutable ephemeral account balance from the delta.
        let ephemeral_account_balance = match self
            .delta_accounts
            .updated_account_balances
            .get_mut(&account_key)
        {
            // If the balance is already in the delta, return it.
            Some(value) => value,
            // Otherwise, from the permanent in-memory states.
            None => {
                // Get the mutable account body from the permanent states.
                let account_body = self.in_memory_accounts.get(&account_key).ok_or(
                    CHAccountBalanceUpError::UnableToGetAccountBalance(account_key),
                )?;

                // Insert the account balance into the delta.
                self.delta_accounts
                    .updated_account_balances
                    .insert(account_key, account_body.balance);

                // Get the mutable ephemeral account balance from the delta that we just inserted.
                let ephemeral_account_balance = self
                    .delta_accounts
                    .updated_account_balances
                    .get_mut(&account_key)
                    .expect("This cannot happen because we just inserted it.");

                // Return the ephemeral account balance.
                ephemeral_account_balance
            }
        };

        // Update the account balance.
        *ephemeral_account_balance = new_account_balance_in_satoshis;

        // Return the result.
        Ok(())
    }

    /// Decreases the account's balance.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn account_balance_down(
        &mut self,
        account_key: ACCOUNT_KEY,
        down_value_in_satoshis: u64,
    ) -> Result<(), CHAccountBalanceDownError> {
        // Get the old ephemeral account balance before any mutable borrows.
        let existing_account_balance_in_satoshis: u64 =
            self.get_account_balance(account_key).ok_or(
                CHAccountBalanceDownError::UnableToGetAccountBalance(account_key),
            )?;

        // Check if the decrease would make the account balance go below zero.
        if down_value_in_satoshis > existing_account_balance_in_satoshis {
            return Err(CHAccountBalanceDownError::AccountBalanceWouldGoBelowZero(
                account_key,
                existing_account_balance_in_satoshis,
                down_value_in_satoshis,
            ));
        }

        // Calculate the new ephemeral account balance.
        let new_account_balance_in_satoshis: u64 =
            existing_account_balance_in_satoshis - down_value_in_satoshis;

        // Retrieve the mutable ephemeral account balance from the delta.
        let ephemeral_account_balance = match self
            .delta_accounts
            .updated_account_balances
            .get_mut(&account_key)
        {
            // If the ephemeral account balance is already in the delta, return it.
            Some(value) => value,
            // Otherwise, from the permanent in-memory states.
            None => {
                // Get the mutable account body from the permanent states.
                let account_body = self.in_memory_accounts.get(&account_key).ok_or(
                    CHAccountBalanceDownError::UnableToGetAccountBalance(account_key),
                )?;

                // Insert the account balance into the delta.
                self.delta_accounts
                    .updated_account_balances
                    .insert(account_key, account_body.balance);

                // Get the mutable ephemeral account balance from the delta that we just inserted.
                let ephemeral_account_balance = self
                    .delta_accounts
                    .updated_account_balances
                    .get_mut(&account_key)
                    .expect("This cannot happen because we just inserted it.");

                // Return the ephemeral account balance.
                ephemeral_account_balance
            }
        };

        // Update the ephemeral account balance.
        *ephemeral_account_balance = new_account_balance_in_satoshis;

        // Return the result.
        Ok(())
    }

    /// Increases contract's balance.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn contract_balance_up(
        &mut self,
        contract_id: [u8; 32],
        up_value_in_satoshis: u64,
    ) -> Result<(), CHContractBalanceUpError> {
        // Get the old contract balance before any mutable borrows.
        let existing_contract_balance_in_satoshis: u64 =
            self.get_contract_balance(contract_id).ok_or(
                CHContractBalanceUpError::UnableToGetContractBalance(contract_id),
            )?;

        // Calculate the new contract balance.
        let new_contract_balance_in_satoshis: u64 =
            existing_contract_balance_in_satoshis + up_value_in_satoshis;

        // Retrieve the mutable balance from the ephemeral states.
        let ephemeral_contract_balance = match self
            .delta_contracts
            .updated_contract_balances
            .get_mut(&contract_id)
        {
            // If the balance is already in the ephemeral states, return it.
            Some(balance) => balance,
            // Otherwise, from the permanent in-memory states.
            None => {
                // Get the mutable balance from the permanent in-memory states.
                let contract_body = self.in_memory_contracts.get_mut(&contract_id).ok_or(
                    CHContractBalanceUpError::UnableToGetContractBody(contract_id),
                )?;

                // Get the mutable balance.
                let balance = contract_body.balance;

                // Insert the balance into the ephemeral states.
                self.delta_contracts
                    .updated_contract_balances
                    .insert(contract_id, balance);

                // Get the mutable balance from the ephemeral that we just inserted.
                let ephemeral_balance = self
                    .delta_contracts
                    .updated_contract_balances
                    .get_mut(&contract_id)
                    .expect("This cannot happen because we just inserted it.");

                // Return the balance.
                ephemeral_balance
            }
        };

        // Update the contract balance.
        *ephemeral_contract_balance = new_contract_balance_in_satoshis;

        // Return the result.
        Ok(())
    }

    /// Decreases contract's balance.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn contract_balance_down(
        &mut self,
        contract_id: [u8; 32],
        down_value_in_satoshis: u64,
    ) -> Result<(), CHContractBalanceDownError> {
        // Get the old contract balance before any mutable borrows.
        let existing_contract_balance_in_satoshis: u64 =
            self.get_contract_balance(contract_id).ok_or(
                CHContractBalanceDownError::UnableToGetContractBalance(contract_id),
            )?;

        // Check if the decrease would make the contract balance go below zero.
        if down_value_in_satoshis > existing_contract_balance_in_satoshis {
            return Err(CHContractBalanceDownError::ContractBalanceWouldGoBelowZero(
                contract_id,
                existing_contract_balance_in_satoshis,
                down_value_in_satoshis,
            ));
        }

        // Calculate the new contract balance.
        let new_contract_balance_in_satoshis: u64 =
            existing_contract_balance_in_satoshis - down_value_in_satoshis;

        // Retrieve the mutable balance from the delta.
        let ephemeral_contract_balance = match self
            .delta_contracts
            .updated_contract_balances
            .get_mut(&contract_id)
        {
            // If the balance is already in the delta, return it.
            Some(balance) => balance,
            // Otherwise, from the permanent in-memory states.
            None => {
                // Get the mutable balance from the permanent in-memory states.
                let contract_body = self.in_memory_contracts.get_mut(&contract_id).ok_or(
                    CHContractBalanceDownError::UnableToGetContractBody(contract_id),
                )?;

                // Get the mutable balance.
                let balance = contract_body.balance;

                // Insert the balance into the delta.
                self.delta_contracts
                    .updated_contract_balances
                    .insert(contract_id, balance);

                // Get the mutable balance from the delta that we just inserted.
                let ephemeral_balance = self
                    .delta_contracts
                    .updated_contract_balances
                    .get_mut(&contract_id)
                    .expect("This cannot happen because we just inserted it.");

                // Return the balance.
                ephemeral_balance
            }
        };

        // Update the contract balance.
        *ephemeral_contract_balance = new_contract_balance_in_satoshis;

        // Return the result.
        Ok(())
    }

    /// Allocates a new account in the contract's shadow space.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn shadow_alloc_account(
        &mut self,
        contract_id: [u8; 32],
        account_key: ACCOUNT_KEY,
    ) -> Result<(), CHContractShadowAllocAccountError> {
        // Check if the account has just been epheremally allocated in the delta.
        // We do not allow it to be allocated again in the same execution.
        if let Some(allocs_list) = self.delta_contracts.allocs_list.get(&contract_id) {
            if allocs_list.contains(&account_key) {
                return Err(
                    CHContractShadowAllocAccountError::AccountHasJustBeenEphemerallyAllocated(
                        contract_id,
                        account_key,
                    ),
                );
            }
        }

        // Check if the account has just been epheremally deallocated in the delta.
        // We do not allow it to be allocated after being deallocated in the same execution.
        if let Some(deallocs_list) = self.delta_contracts.deallocs_list.get(&contract_id) {
            if deallocs_list.contains(&account_key) {
                return Err(
                    CHContractShadowAllocAccountError::AccountHasJustBeenEphemerallyDeallocated(
                        contract_id,
                        account_key,
                    ),
                );
            }
        }

        // Check if the account key is already permanently allocated by reading its allocation value.
        // We do not allow it to be allocated again if already permanently allocated.
        if self
            .get_account_shadow_alloc_value_of_a_contract_in_sati_satoshis(contract_id, account_key)
            .is_some()
        {
            return Err(
                CHContractShadowAllocAccountError::UnableToGetAccountAllocValue(
                    contract_id,
                    account_key,
                ),
            );
        }

        // Retrieve the mutable shadow space from the delta.
        let epheremal_shadow_space = match self
            .delta_contracts
            .updated_shadow_spaces
            .get_mut(&contract_id)
        {
            Some(shadow_space) => shadow_space,
            None => {
                // Otherwise, from the permanent states.
                let contract_body = self.in_memory_contracts.get(&contract_id).ok_or(
                    CHContractShadowAllocAccountError::UnableToGetContractBody(contract_id),
                )?;

                // Clone the shadow space from permanent states.
                let shadow_space = contract_body.shadow_space.clone();

                // Insert the shadow space into the delta.
                self.delta_contracts
                    .updated_shadow_spaces
                    .insert(contract_id, shadow_space);

                // Get the mutable shadow space from the delta that we just inserted.
                let delta_shadow_space = self
                    .delta_contracts
                    .updated_shadow_spaces
                    .get_mut(&contract_id)
                    .expect("This cannot happen because we just inserted it.");

                // Return the mutable shadow space.
                delta_shadow_space
            }
        };

        // Insert the account key into the shadow space in the delta with the value initalliy set to zero.
        epheremal_shadow_space.allocs.insert(account_key, 0);

        // Insert the account key into the allocs list in the delta.
        self.delta_contracts
            .allocs_list
            .entry(contract_id)
            .or_insert_with(Vec::new)
            .push(account_key);

        // Return the result.
        Ok(())
    }

    /// Deallocates an account from the contract's shadow space.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn shadow_dealloc_account(
        &mut self,
        contract_id: [u8; 32],
        account_key: ACCOUNT_KEY,
    ) -> Result<(), CHContractShadowDeallocAccountError> {
        // Check if the account has just been epheremally allocated in the delta.
        // We do not allow it to be deallocated if it is just allocated in the same execution.
        if self
            .delta_contracts
            .allocs_list
            .get(&contract_id)
            .unwrap_or(&Vec::new())
            .contains(&account_key)
        {
            return Err(
                CHContractShadowDeallocAccountError::AccountHasJustBeenEphemerallyAllocated(
                    contract_id,
                    account_key,
                ),
            );
        }

        // Get the account's allocation value in sati-satoshis.
        // This also checks if the account is acutally permanently allocated.
        let allocation_value_in_sati_satoshis = self
            .get_account_shadow_alloc_value_of_a_contract_in_sati_satoshis(contract_id, account_key)
            .ok_or(
                CHContractShadowDeallocAccountError::UnableToGetAccountAllocValue(
                    contract_id,
                    account_key,
                ),
            )?;

        // Check if the account allocation value is non-zero.
        // Deallocation is allowed only if the allocation value is zero.
        if allocation_value_in_sati_satoshis != 0 {
            return Err(CHContractShadowDeallocAccountError::AllocValueIsNonZero(
                contract_id,
                account_key,
            ));
        }

        // Get the mutable epheremal dealloc list from the delta.
        let epheremal_dealloc_list = match self.delta_contracts.deallocs_list.get_mut(&contract_id)
        {
            Some(dealloc_list) => dealloc_list,
            None => {
                // Create a fresh dealloc list.
                let fresh_dealloc_list = Vec::new();

                // Insert the dealloc list into the delta.
                self.delta_contracts
                    .deallocs_list
                    .insert(contract_id, fresh_dealloc_list);

                // Get the mutable dealloc list from the delta that we just inserted.
                let delta_dealloc_list = self
                    .delta_contracts
                    .deallocs_list
                    .get_mut(&contract_id)
                    .expect("This cannot happen because we just inserted it.");

                // Return the mutable dealloc list.
                delta_dealloc_list
            }
        };

        // Check if the account has just been epheremally deallocated in the delta.
        // We do not allow it to be deallocated if it is just deallocated in the same execution.
        if epheremal_dealloc_list.contains(&account_key) {
            return Err(
                CHContractShadowDeallocAccountError::AccountHasJustBeenEphemerallyDeallocated(
                    contract_id,
                    account_key,
                ),
            );
        }

        // Insert the account key into the epheremal dealloc list.
        epheremal_dealloc_list.push(account_key);

        // Get the mutable epheremal shadow space from the delta.
        let epheremal_shadow_space = match self
            .delta_contracts
            .updated_shadow_spaces
            .get_mut(&contract_id)
        {
            Some(shadow_space) => shadow_space,
            None => {
                // Otherwise, from the permanent states.
                let contract_body = self.in_memory_contracts.get(&contract_id).ok_or(
                    CHContractShadowDeallocAccountError::UnableToGetContractBody(contract_id),
                )?;

                // Clone the shadow space from permanent states.
                let shadow_space = contract_body.shadow_space.clone();

                // Insert the shadow space into the delta.
                self.delta_contracts
                    .updated_shadow_spaces
                    .insert(contract_id, shadow_space);

                // Get the mutable shadow space from the delta that we just inserted.
                let delta_shadow_space = self
                    .delta_contracts
                    .updated_shadow_spaces
                    .get_mut(&contract_id)
                    .expect("This cannot happen because we just inserted it.");

                // Return the mutable shadow space.
                delta_shadow_space
            }
        };

        // Remove the account key from the shadow space in the delta.
        epheremal_shadow_space.allocs.remove(&account_key);

        // Return the result.
        Ok(())
    }

    /// Increases the account's individual shadow allocs sum value.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    fn account_shadow_allocs_sum_up(
        &mut self,
        account_key: ACCOUNT_KEY,
        up_value_in_sati_satoshis: u128,
    ) -> Result<(), CHAccountShadowAllocsSumUpError> {
        // Get the old ephemeral account shadow allocs sum before any mutable borrows.
        let existing_account_shadow_allocs_sum_in_sati_satoshis: u128 = self
            .get_account_shadow_allocs_sum_of_all_contracts_in_sati_satoshis(account_key)
            .ok_or(
                CHAccountShadowAllocsSumUpError::UnableToGetAccountShadowAllocsSum(account_key),
            )?;

        // Calculate the new ephemeral account shadow allocs sum.
        let new_account_shadow_allocs_sum_in_sati_satoshis: u128 =
            existing_account_shadow_allocs_sum_in_sati_satoshis + up_value_in_sati_satoshis;

        // Retrieve the mutable ephemeral account shadow allocs sum from the delta.
        let ephemeral_account_shadow_allocs_sum = match self
            .delta_accounts
            .updated_shadow_allocs_sums
            .get_mut(&account_key)
        {
            // If the ephemeral account shadow allocs sum is already in the delta, return it.
            Some(value) => value,
            // Otherwise, from the permanent in-memory states.
            None => {
                // Get the mutable account body from the permanent states.
                let account_body = self.in_memory_accounts.get(&account_key).ok_or(
                    CHAccountShadowAllocsSumUpError::UnableToGetAccountBody(account_key),
                )?;

                // Insert the account shadow allocs sum into the delta.
                self.delta_accounts
                    .updated_shadow_allocs_sums
                    .insert(account_key, account_body.shadow_allocs_sum);

                // Get the mutable ephemeral account shadow allocs sum from the delta that we just inserted.
                let ephemeral_account_shadow_allocs_sum = self
                    .delta_accounts
                    .updated_shadow_allocs_sums
                    .get_mut(&account_key)
                    .expect("This cannot happen because we just inserted it.");

                // Return the ephemeral account shadow allocs sum.
                ephemeral_account_shadow_allocs_sum
            }
        };

        // Update the ephemeral account shadow allocs sum.
        *ephemeral_account_shadow_allocs_sum = new_account_shadow_allocs_sum_in_sati_satoshis;

        // Return the result.
        Ok(())
    }

    /// Decreases the account's individual shadow allocs sum value.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    fn account_shadow_allocs_sum_down(
        &mut self,
        account_key: ACCOUNT_KEY,
        down_value_in_sati_satoshis: u128,
    ) -> Result<(), CHAccountShadowAllocsSumDownError> {
        // Get the old ephemeral account shadow allocs sum before any mutable borrows.
        let existing_account_shadow_allocs_sum_in_sati_satoshis: u128 = self
            .get_account_shadow_allocs_sum_of_all_contracts_in_sati_satoshis(account_key)
            .ok_or(
                CHAccountShadowAllocsSumDownError::UnableToGetAccountShadowAllocsSum(account_key),
            )?;

        // Check if the decrease would make the account shadow allocs sum go below zero.
        if down_value_in_sati_satoshis > existing_account_shadow_allocs_sum_in_sati_satoshis {
            return Err(
                CHAccountShadowAllocsSumDownError::AccountShadowAllocsSumWouldGoBelowZero(
                    account_key,
                    existing_account_shadow_allocs_sum_in_sati_satoshis,
                    down_value_in_sati_satoshis,
                ),
            );
        }

        // Calculate the new ephemeral account shadow allocs sum.
        let new_account_shadow_allocs_sum_in_sati_satoshis: u128 =
            existing_account_shadow_allocs_sum_in_sati_satoshis - down_value_in_sati_satoshis;

        // Retrieve the mutable ephemeral account shadow allocs sum from the delta.
        let ephemeral_account_shadow_allocs_sum = match self
            .delta_accounts
            .updated_shadow_allocs_sums
            .get_mut(&account_key)
        {
            // If the ephemeral account shadow allocs sum is already in the delta, return it.
            Some(value) => value,
            // Otherwise, from the permanent in-memory states.
            None => {
                // Get the mutable account body from the permanent states.
                let account_body = self.in_memory_accounts.get(&account_key).ok_or(
                    CHAccountShadowAllocsSumDownError::UnableToGetAccountBody(account_key),
                )?;

                // Insert the account shadow allocs sum into the delta.
                self.delta_accounts
                    .updated_shadow_allocs_sums
                    .insert(account_key, account_body.shadow_allocs_sum);

                // Get the mutable ephemeral account shadow allocs sum from the delta that we just inserted.
                let ephemeral_account_shadow_allocs_sum = self
                    .delta_accounts
                    .updated_shadow_allocs_sums
                    .get_mut(&account_key)
                    .expect("This cannot happen because we just inserted it.");

                // Return the ephemeral account shadow allocs sum.
                ephemeral_account_shadow_allocs_sum
            }
        };

        // Update the ephemeral account shadow allocs sum.
        *ephemeral_account_shadow_allocs_sum = new_account_shadow_allocs_sum_in_sati_satoshis;

        // Return the result.
        Ok(())
    }

    /// Increases an account's shadow allocation value in the contract's shadow space.    
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn shadow_up(
        &mut self,
        contract_id: [u8; 32],
        account_key: ACCOUNT_KEY,
        up_value_in_satoshis: u64,
    ) -> Result<(), CHShadowUpError> {
        // Convert the increase value to sati-satoshi value.
        let up_value_in_sati_satoshis: u128 = (up_value_in_satoshis as u128) * 100_000_000;

        // Get the old account allocation value and contract balance before any mutable borrows.
        let existing_account_shadow_alloc_value_in_sati_satoshis: u128 = self
            .get_account_shadow_alloc_value_of_a_contract_in_sati_satoshis(contract_id, account_key)
            .ok_or(CHShadowUpError::UnableToGetAccountShadowAllocValue(
                contract_id,
                account_key,
            ))?;

        // Get existing contract balance.
        let existing_contract_balance_in_satoshis: u64 = self
            .get_contract_balance(contract_id)
            .ok_or(CHShadowUpError::UnableToGetContractBalance(contract_id))?;

        // Retrieve the mutable shadow space from the delta.
        let epheremal_shadow_space = match self
            .delta_contracts
            .updated_shadow_spaces
            .get_mut(&contract_id)
        {
            // If the shadow space is already in the ephemeral states, return it.
            Some(shadow_space) => shadow_space,
            // Otherwise, from the permanent in-memory states.
            None => {
                let contract_body = self
                    .in_memory_contracts
                    .get_mut(&contract_id)
                    .ok_or(CHShadowUpError::UnableToGetContractBody(contract_id))?;

                // Get the mutable shadow space.
                let shadow_space = contract_body.shadow_space.clone();

                // Insert the shadow space into the ephemeral states.
                self.delta_contracts
                    .updated_shadow_spaces
                    .insert(contract_id, shadow_space);

                // Get the mutable shadow space from the epheremal that we just inserted.
                let epheremal_shadow_space = self
                    .delta_contracts
                    .updated_shadow_spaces
                    .get_mut(&contract_id)
                    .expect("This cannot happen because we just inserted it.");

                // Return the shadow space.
                epheremal_shadow_space
            }
        };

        // Calculate the new allocation value.
        let new_account_shadow_alloc_value_in_sati_satoshis: u128 =
            existing_account_shadow_alloc_value_in_sati_satoshis + up_value_in_sati_satoshis;

        // Calculate the new allocation sum value.
        let new_contract_allocs_sum_value_in_satoshis: u64 =
            epheremal_shadow_space.allocs_sum + up_value_in_satoshis;

        // Check if the new contract alloc sum value exceeds the contract balance.
        if new_contract_allocs_sum_value_in_satoshis > existing_contract_balance_in_satoshis {
            return Err(CHShadowUpError::AllocsSumExceedsTheContractBalance(
                contract_id,
                new_contract_allocs_sum_value_in_satoshis,
                existing_contract_balance_in_satoshis,
            ));
        }

        // Epheremally update the account shadow allocation value.
        epheremal_shadow_space
            .allocs
            .insert(account_key, new_account_shadow_alloc_value_in_sati_satoshis);

        // Epheremally update the contract shadow allocation sum value.
        epheremal_shadow_space.allocs_sum = new_contract_allocs_sum_value_in_satoshis;

        // Update the account shadow allocs sum value.
        {
            self.account_shadow_allocs_sum_up(account_key, up_value_in_sati_satoshis)
                .map_err(|error| {
                    CHShadowUpError::AccountShadowAllocsSumUpError(contract_id, account_key, error)
                })?;
        }

        // Return the result.
        Ok(())
    }

    /// Decreases a shadow allocation value by key and contract ID ephemerally.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn shadow_down(
        &mut self,
        contract_id: [u8; 32],
        account_key: ACCOUNT_KEY,
        down_value_in_satoshis: u64,
    ) -> Result<(), CHShadowDownError> {
        // Convert the decrease value to sati-satoshi value.
        let down_value_in_sati_satoshis: u128 = (down_value_in_satoshis as u128) * 100_000_000;

        // Get the old account allocation value and contract balance before any mutable borrows.
        let existing_account_shadow_alloc_value_in_sati_satoshis: u128 = self
            .get_account_shadow_alloc_value_of_a_contract_in_sati_satoshis(contract_id, account_key)
            .ok_or(CHShadowDownError::UnableToGetAccountShadowAllocValue(
                contract_id,
                account_key,
            ))?;

        // Get existing contract balance.
        let existing_contract_balance_in_satoshis: u64 = self
            .get_contract_balance(contract_id)
            .ok_or(CHShadowDownError::UnableToGetContractBalance(contract_id))?;

        // Check if the decrease would make the allocation value go below zero.
        if down_value_in_sati_satoshis > existing_account_shadow_alloc_value_in_sati_satoshis {
            return Err(CHShadowDownError::AccountShadowAllocValueWouldGoBelowZero(
                contract_id,
                account_key,
                existing_account_shadow_alloc_value_in_sati_satoshis,
                down_value_in_sati_satoshis,
            ));
        }

        // Retrieve the mutable shadow space from the delta.
        let epheremal_shadow_space = match self
            .delta_contracts
            .updated_shadow_spaces
            .get_mut(&contract_id)
        {
            // If the shadow space is already in the delta, return it.
            Some(shadow_space) => shadow_space,
            // Otherwise, from the permanent in-memory states.
            None => {
                let contract_body = self
                    .in_memory_contracts
                    .get_mut(&contract_id)
                    .ok_or(CHShadowDownError::UnableToGetContractBody(contract_id))?;

                // Get the mutable shadow space.
                let shadow_space = contract_body.shadow_space.clone();

                // Insert the shadow space into the delta.
                self.delta_contracts
                    .updated_shadow_spaces
                    .insert(contract_id, shadow_space);

                // Get the mutable shadow space from the epheremal that we just inserted.
                let epheremal_shadow_space = self
                    .delta_contracts
                    .updated_shadow_spaces
                    .get_mut(&contract_id)
                    .expect("This cannot happen because we just inserted it.");

                // Return the shadow space.
                epheremal_shadow_space
            }
        };

        // Calculate the new allocation value.
        let new_account_shadow_alloc_value_in_sati_satoshis: u128 =
            existing_account_shadow_alloc_value_in_sati_satoshis - down_value_in_sati_satoshis;

        // Calculate the new allocation sum value.
        let new_contract_allocs_sum_value_in_satoshis: u64 =
            epheremal_shadow_space.allocs_sum - down_value_in_satoshis;

        // Check if the new contract alloc sum value exceeds the contract balance.
        if new_contract_allocs_sum_value_in_satoshis > existing_contract_balance_in_satoshis {
            return Err(CHShadowDownError::AllocsSumExceedsTheContractBalance(
                contract_id,
                new_contract_allocs_sum_value_in_satoshis,
                existing_contract_balance_in_satoshis,
            ));
        }

        // Epheremally update the account shadow allocation value.
        epheremal_shadow_space
            .allocs
            .insert(account_key, new_account_shadow_alloc_value_in_sati_satoshis);

        // Epheremally update the contract shadow allocation sum value.
        epheremal_shadow_space.allocs_sum = new_contract_allocs_sum_value_in_satoshis;

        // Update the account shadow allocs sum value.
        {
            self.account_shadow_allocs_sum_down(account_key, down_value_in_sati_satoshis)
                .map_err(|error| {
                    CHShadowDownError::AccountShadowAllocsSumDownError(
                        contract_id,
                        account_key,
                        error,
                    )
                })?;
        }

        // Return the result.
        Ok(())
    }

    /// Proportionaly increases the shadow allocation value of all accounts in a contract shadow space by a given value.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn shadow_up_all(
        &mut self,
        contract_id: [u8; 32],
        up_value_in_satoshis: u64,
    ) -> Result<u64, CHShadowUpAllError> {
        // Convert the increase value to sati-satoshi value.
        let up_value_in_sati_satoshis: u128 = (up_value_in_satoshis as u128) * 100_000_000;

        // Get the old contract balance and allocs sum before any mutable borrows.
        let contract_balance_in_satoshis: u64 = self
            .get_contract_balance(contract_id)
            .ok_or(CHShadowUpAllError::UnableToGetContractBalance(contract_id))?;

        // Get the old contract allocs sum before any mutable borrows.
        let existing_contract_allocs_sum_in_satoshis: u64 = self
            .get_contract_allocs_sum_in_satoshis(contract_id)
            .ok_or(CHShadowUpAllError::UnableToGetContractAllocsSum(
                contract_id,
            ))?;

        // Check if the contract allocs sum is zero.
        // This operation is not possible with zero allocs sum.
        if existing_contract_allocs_sum_in_satoshis == 0 {
            return Err(CHShadowUpAllError::OperationNotPossibleWithZeroAllocsSum(
                contract_id,
            ));
        }

        // Calculate the new contract allocs sum value.
        let new_contract_allocs_sum_value_in_satoshis: u64 =
            existing_contract_allocs_sum_in_satoshis + up_value_in_satoshis;

        // Check if the new contract alloc sum value exceeds the contract balance.
        if new_contract_allocs_sum_value_in_satoshis > contract_balance_in_satoshis {
            return Err(CHShadowUpAllError::AllocsSumExceedsTheContractBalance(
                contract_id,
                new_contract_allocs_sum_value_in_satoshis,
                contract_balance_in_satoshis,
            ));
        }

        // Convert the old contract allocs sum to sati-satoshi value.
        let existing_contract_allocs_sum_in_satisatoshis: u128 =
            (existing_contract_allocs_sum_in_satoshis as u128) * 100_000_000;

        // Initialize a list of update values of individual accounts.
        // (up value, updated value)
        let mut individual_update_values_in_sati_satoshis: HashMap<ACCOUNT_KEY, (u128, u128)> =
            HashMap::new();

        // Iterate over all all account in the shadow space.
        for (account_key, shadow_alloc_value_in_sati_satoshis) in match self
            .delta_contracts
            .updated_shadow_spaces
            .get_mut(&contract_id)
        {
            // First try the ephemeral shadow space.
            Some(shadow_space) => shadow_space.allocs.iter(),
            // Otherwise from the in-memory shadow space.
            None => self
                .in_memory_contracts
                .get_mut(&contract_id)
                .ok_or(CHShadowUpAllError::UnableToGetContractBody(contract_id))?
                .shadow_space
                .allocs
                .iter(),
        } {
            // shadow_alloc_value_in_sati_satoshis divided by existing_contract_allocs_sum_in_satisatoshis = x divided by up_value_in_sati_satoshis.
            // NOTE: if the account is ephemerally deallocated, since it's allocation value had to be zero, this will also be zero.
            let individual_up_value_in_sati_satoshis: u128 = (shadow_alloc_value_in_sati_satoshis
                * up_value_in_sati_satoshis)
                / existing_contract_allocs_sum_in_satisatoshis;

            // If the individual up value is greater than zero, insert it into the list of new values.
            if individual_up_value_in_sati_satoshis > 0 {
                // Calculate the new value.
                let individual_new_value_in_sati_satoshis: u128 =
                    shadow_alloc_value_in_sati_satoshis + individual_up_value_in_sati_satoshis;

                // Insert the new value into the list of update values.
                individual_update_values_in_sati_satoshis.insert(
                    *account_key,
                    (
                        individual_up_value_in_sati_satoshis,
                        individual_new_value_in_sati_satoshis,
                    ),
                );
            }
        }

        // Retrieve the mutable shadow space from the delta.
        let epheremal_shadow_space = match self
            .delta_contracts
            .updated_shadow_spaces
            .get_mut(&contract_id)
        {
            // If the shadow space is already in the delta, return it.
            Some(shadow_space) => shadow_space,
            // Otherwise, from the permanent in-memory states.
            None => {
                let contract_body = self
                    .in_memory_contracts
                    .get_mut(&contract_id)
                    .ok_or(CHShadowUpAllError::UnableToGetContractBody(contract_id))?;

                // Get the mutable shadow space.
                let shadow_space = contract_body.shadow_space.clone();

                // Insert the shadow space into the delta.
                self.delta_contracts
                    .updated_shadow_spaces
                    .insert(contract_id, shadow_space);

                // Get the mutable shadow space from the epheremal that we just inserted.
                let epheremal_shadow_space = self
                    .delta_contracts
                    .updated_shadow_spaces
                    .get_mut(&contract_id)
                    .expect("This cannot happen because we just inserted it.");

                // Return the shadow space.
                epheremal_shadow_space
            }
        };

        // Epheremally update the account shadow allocation value.
        for (account_key, (_, individual_new_value_in_sati_satoshis)) in
            individual_update_values_in_sati_satoshis.iter()
        {
            epheremal_shadow_space
                .allocs
                .insert(*account_key, *individual_new_value_in_sati_satoshis);
        }

        // Update the allocs sum value in the ephemeral shadow space.
        epheremal_shadow_space.allocs_sum = new_contract_allocs_sum_value_in_satoshis;

        // Update the account shadow allocs sum value.
        for (account_key, (individual_up_value_in_sati_satoshis, _)) in
            individual_update_values_in_sati_satoshis.iter()
        {
            self.account_shadow_allocs_sum_up(*account_key, *individual_up_value_in_sati_satoshis)
                .map_err(|error| {
                    CHShadowUpAllError::AccountShadowAllocsSumUpError(
                        contract_id,
                        *account_key,
                        error,
                    )
                })?;
        }

        // Return the number of updated accounts.
        Ok(individual_update_values_in_sati_satoshis.len() as u64)
    }

    /// Proportionaly decreases the shadow allocation value of all accounts in a contract shadow space by a given value.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn shadow_down_all(
        &mut self,
        contract_id: [u8; 32],
        down_value_in_satoshis: u64,
    ) -> Result<u64, CHShadowDownAllError> {
        // Convert the decrease value to sati-satoshi value.
        let down_value_in_sati_satoshis: u128 = (down_value_in_satoshis as u128) * 100_000_000;

        // Get the old contract balance and allocs sum before any mutable borrows.
        let contract_balance_in_satoshis: u64 = self.get_contract_balance(contract_id).ok_or(
            CHShadowDownAllError::UnableToGetContractBalance(contract_id),
        )?;

        // Get the old contract allocs sum before any mutable borrows.
        let existing_contract_allocs_sum_in_satoshis: u64 = self
            .get_contract_allocs_sum_in_satoshis(contract_id)
            .ok_or(CHShadowDownAllError::UnableToGetContractAllocsSum(
                contract_id,
            ))?;

        // Check if the contract allocs sum is zero.
        // This operation is not possible with zero allocs sum.
        if existing_contract_allocs_sum_in_satoshis == 0 {
            return Err(CHShadowDownAllError::OperationNotPossibleWithZeroAllocsSum(
                contract_id,
            ));
        }

        // Check if would go below zero.
        if down_value_in_satoshis > existing_contract_allocs_sum_in_satoshis {
            return Err(CHShadowDownAllError::AllocsSumWouldGoBelowZero(
                contract_id,
                existing_contract_allocs_sum_in_satoshis,
                down_value_in_satoshis,
            ));
        }

        // Calculate the new contract allocs sum value.
        let new_contract_allocs_sum_value_in_satoshis: u64 =
            existing_contract_allocs_sum_in_satoshis - down_value_in_satoshis;

        // Check if the new contract alloc sum value exceeds the contract balance.
        if new_contract_allocs_sum_value_in_satoshis > contract_balance_in_satoshis {
            return Err(CHShadowDownAllError::AllocsSumExceedsTheContractBalance(
                contract_id,
                new_contract_allocs_sum_value_in_satoshis,
                contract_balance_in_satoshis,
            ));
        }

        // Convert the old contract allocs sum to sati-satoshi value.
        let existing_contract_allocs_sum_in_satisatoshis: u128 =
            (existing_contract_allocs_sum_in_satoshis as u128) * 100_000_000;

        // Initialize a list of update values of individual accounts.
        // (down value, updated value)
        let mut individual_update_values_in_sati_satoshis: HashMap<ACCOUNT_KEY, (u128, u128)> =
            HashMap::new();

        // Iterate over all all account in the shadow space.
        for (account_key, shadow_alloc_value_in_sati_satoshis) in match self
            .delta_contracts
            .updated_shadow_spaces
            .get_mut(&contract_id)
        {
            // First try the ephemeral shadow space.
            Some(shadow_space) => shadow_space.allocs.iter(),
            // Otherwise from the in-memory shadow space.
            None => self
                .in_memory_contracts
                .get_mut(&contract_id)
                .ok_or(CHShadowDownAllError::UnableToGetContractBody(contract_id))?
                .shadow_space
                .allocs
                .iter(),
        } {
            // shadow_alloc_value_in_sati_satoshis divided by existing_contract_allocs_sum_in_satisatoshis = x divided by down_value_in_sati_satoshis.
            // NOTE: if the account is ephemerally deallocated, since it's allocation value had to be zero, this will also be zero.
            let individual_down_value_in_sati_satoshis: u128 = (shadow_alloc_value_in_sati_satoshis
                * down_value_in_sati_satoshis)
                / existing_contract_allocs_sum_in_satisatoshis;

            // Check if the individual down value would go below zero.
            if individual_down_value_in_sati_satoshis > *shadow_alloc_value_in_sati_satoshis {
                return Err(
                    CHShadowDownAllError::AccountShadowAllocValueWouldGoBelowZero(
                        contract_id,
                        *account_key,
                        *shadow_alloc_value_in_sati_satoshis,
                        individual_down_value_in_sati_satoshis,
                    ),
                );
            }

            // If the individual up value is greater than zero, insert it into the list of new values.
            if individual_down_value_in_sati_satoshis > 0 {
                // Calculate the new value.
                let individual_new_value_in_sati_satoshis: u128 =
                    shadow_alloc_value_in_sati_satoshis - individual_down_value_in_sati_satoshis;

                // Insert the new value into the list of update values.
                individual_update_values_in_sati_satoshis.insert(
                    *account_key,
                    (
                        individual_down_value_in_sati_satoshis,
                        individual_new_value_in_sati_satoshis,
                    ),
                );
            }
        }

        // Retrieve the mutable shadow space from the delta.
        let epheremal_shadow_space = match self
            .delta_contracts
            .updated_shadow_spaces
            .get_mut(&contract_id)
        {
            // If the shadow space is already in the delta, return it.
            Some(shadow_space) => shadow_space,
            // Otherwise, from the permanent in-memory states.
            None => {
                let contract_body = self
                    .in_memory_contracts
                    .get_mut(&contract_id)
                    .ok_or(CHShadowDownAllError::UnableToGetContractBody(contract_id))?;

                // Get the mutable shadow space.
                let shadow_space = contract_body.shadow_space.clone();

                // Insert the shadow space into the delta.
                self.delta_contracts
                    .updated_shadow_spaces
                    .insert(contract_id, shadow_space);

                // Get the mutable shadow space from the epheremal that we just inserted.
                let epheremal_shadow_space = self
                    .delta_contracts
                    .updated_shadow_spaces
                    .get_mut(&contract_id)
                    .expect("This cannot happen because we just inserted it.");

                // Return the shadow space.
                epheremal_shadow_space
            }
        };

        // Epheremally update the account shadow allocation value.
        for (account_key, (_, individual_new_value_in_sati_satoshis)) in
            individual_update_values_in_sati_satoshis.iter()
        {
            epheremal_shadow_space
                .allocs
                .insert(*account_key, *individual_new_value_in_sati_satoshis);
        }

        // Update the allocs sum value in the ephemeral shadow space.
        epheremal_shadow_space.allocs_sum = new_contract_allocs_sum_value_in_satoshis;

        // Update the account shadow allocs sum value.
        for (account_key, (individual_down_value_in_sati_satoshis, _)) in
            individual_update_values_in_sati_satoshis.iter()
        {
            self.account_shadow_allocs_sum_down(
                *account_key,
                *individual_down_value_in_sati_satoshis,
            )
            .map_err(|error| {
                CHShadowDownAllError::AccountShadowAllocsSumDownError(
                    contract_id,
                    *account_key,
                    error,
                )
            })?;
        }

        // Return the number of updated accounts.
        Ok(individual_update_values_in_sati_satoshis.len() as u64)
    }

    /// Reverts the epheremal changes associated with the last execution.
    pub fn rollback_last(&mut self) {
        // Restore the ephemeral states from the backup.
        self.restore_delta();
    }

    /// Clears all epheremal changes from the delta.
    pub fn flush_delta(&mut self) {
        // Clear the ephemeral states.
        self.delta_accounts.flush();
        self.delta_contracts.flush();

        // Clear the ephemeral states backup.
        self.backup_of_delta_accounts.flush();
        self.backup_of_delta_contracts.flush();
    }

    /// Applies all epheremal changes from the delta into the in-memory and on-disk.
    pub fn apply_changes(&mut self) -> Result<(), CHApplyChangesError> {
        // 1. Register new accounts.
        for account_key in self.delta_accounts.new_accounts_to_register.iter() {
            // In-memory insertion.
            {
                // Construct the fresh new account body.
                let fresh_new_account_body = CHAccountBody {
                    balance: 0,
                    shadow_allocs_sum: 0,
                };

                // Insert the account balance into the in-memory list.
                // Register the account in-memory with zero balance.
                self.in_memory_accounts
                    .insert(*account_key, fresh_new_account_body);
            }

            // On-disk insertion.
            {
                // Construct the fresh new concatenated value bytes.
                let fresh_new_concatenated_value_bytes: [u8; 24] = [0x00u8; 24];

                // Insert the account balance into the on-disk list.
                self.on_disk_accounts
                    .insert(account_key, fresh_new_concatenated_value_bytes.to_vec())
                    .map_err(|e| {
                        CHApplyChangesError::AccountApplyChangesError(
                            CHAccountApplyChangesError::TreeValueInsertError(
                                account_key.to_owned(),
                                0,
                                e,
                            ),
                        )
                    })?;
            }
        }

        // 2. Register new contracts.
        for contract_id in self.delta_contracts.new_contracts_to_register.iter() {
            // In-memory insertion.
            {
                // Construct the fresh new shadow space.
                let fresh_new_shadow_space = ShadowSpace {
                    allocs_sum: 0,
                    allocs: HashMap::new(),
                };

                // Construct the fresh new contract body.
                let fresh_new_contract_body = CHContractBody {
                    balance: 0,
                    shadow_space: fresh_new_shadow_space,
                };

                // Insert the contract body into the in-memory list.
                // Register the contract in-memory.
                self.in_memory_contracts
                    .insert(*contract_id, fresh_new_contract_body);
            }

            // On-disk insertion.
            {
                // Open tree
                let tree = self.on_disk_contracts.open_tree(contract_id).map_err(|e| {
                    CHApplyChangesError::ContractApplyChangesError(
                        CHContractApplyChangesError::OpenTreeError(*contract_id, e),
                    )
                })?;

                // Insert the contract body into the on-disk list.
                tree.insert(CONTRACT_BALANCE_SPECIAL_KEY, 0u64.to_le_bytes().to_vec())
                    .map_err(|e| {
                        CHApplyChangesError::ContractApplyChangesError(
                            CHContractApplyChangesError::BalanceValueOnDiskInsertionError(
                                *contract_id,
                                0,
                                e,
                            ),
                        )
                    })?;

                // Insert the shadow space into the on-disk list.
                tree.insert(CONTRACT_ALLOCS_SUM_SPECIAL_KEY, 0u64.to_le_bytes().to_vec())
                    .map_err(|e| {
                        CHApplyChangesError::ContractApplyChangesError(
                            CHContractApplyChangesError::AllocsSumValueOnDiskInsertionError(
                                *contract_id,
                                0,
                                e,
                            ),
                        )
                    })?;
            }
        }

        // 3. Save account balances.
        for (account_key, ephemeral_account_balance) in
            self.delta_accounts.updated_account_balances.iter()
        {
            // 1.0 In-memory insertion.
            {
                // Get the mutable account body from the permanent states.
                let account_body = self.in_memory_accounts.get_mut(account_key).ok_or(
                    CHApplyChangesError::AccountApplyChangesError(
                        CHAccountApplyChangesError::UnableToGetAccountBody(*account_key),
                    ),
                )?;

                // Update the balance in the in-memory states.
                account_body.balance = *ephemeral_account_balance;
            }

            // 1.1 On-disk insertion.
            {
                // Open the account tree using the account key as the tree name.
                let tree = self.on_disk_accounts.open_tree(account_key).map_err(|e| {
                    CHApplyChangesError::AccountApplyChangesError(
                        CHAccountApplyChangesError::OpenTreeError(*account_key, e),
                    )
                })?;

                // Save the balance to the balance db.
                tree.insert(
                    ACCOUNT_BALANCE_SPECIAL_KEY,
                    ephemeral_account_balance.to_le_bytes().to_vec(),
                )
                .map_err(|e| {
                    CHApplyChangesError::AccountApplyChangesError(
                        CHAccountApplyChangesError::AccountBalanceValueOnDiskInsertionError(
                            account_key.to_owned(),
                            *ephemeral_account_balance,
                            e,
                        ),
                    )
                })?;
            }
        }

        // 4. Save contract balances.
        for (contract_id, ephemeral_contract_balance) in
            self.delta_contracts.updated_contract_balances.iter()
        {
            // 1.0 In-memory insertion.
            {
                // Get mutable in-memory permanent contract body.
                let in_memory_permanent_contract_body = self
                    .in_memory_contracts
                    .get_mut(contract_id)
                    .ok_or(CHApplyChangesError::ContractApplyChangesError(
                        CHContractApplyChangesError::UnableToGetContractBody(*contract_id),
                    ))?;

                // Update the balance in the in-memory states.
                in_memory_permanent_contract_body.balance = *ephemeral_contract_balance;
            }

            // 1.1 On-disk insertion.
            {
                // Open tree
                let tree = self.on_disk_contracts.open_tree(contract_id).map_err(|e| {
                    CHApplyChangesError::ContractApplyChangesError(
                        CHContractApplyChangesError::OpenTreeError(*contract_id, e),
                    )
                })?;

                // Save the balance to the balances db.
                tree.insert(
                    CONTRACT_BALANCE_SPECIAL_KEY,
                    ephemeral_contract_balance.to_le_bytes().to_vec(),
                )
                .map_err(|e| {
                    CHApplyChangesError::ContractApplyChangesError(
                        CHContractApplyChangesError::BalanceValueOnDiskInsertionError(
                            *contract_id,
                            *ephemeral_contract_balance,
                            e,
                        ),
                    )
                })?;
            }
        }

        // 5. Save account shadow allocs sums.
        for (account_key, ephemeral_account_shadow_allocs_sum) in
            self.delta_accounts.updated_shadow_allocs_sums.iter()
        {
            // 2.0 In-memory insertion.
            {
                // Get the mutable account body from the permanent states.
                let account_body = self.in_memory_accounts.get_mut(account_key).ok_or(
                    CHApplyChangesError::AccountApplyChangesError(
                        CHAccountApplyChangesError::UnableToGetAccountBody(*account_key),
                    ),
                )?;

                // Update the shadow allocs sum in the in-memory states.
                account_body.shadow_allocs_sum = *ephemeral_account_shadow_allocs_sum;
            }

            // 2.1 On-disk insertion.
            {
                // Open the account tree using the account key as the tree name.
                let tree = self.on_disk_accounts.open_tree(account_key).map_err(|e| {
                    CHApplyChangesError::AccountApplyChangesError(
                        CHAccountApplyChangesError::OpenTreeError(*account_key, e),
                    )
                })?;

                tree.insert(
                    ACCOUNT_ALLOCS_SUM_SPECIAL_KEY,
                    ephemeral_account_shadow_allocs_sum.to_le_bytes().to_vec(),
                )
                .map_err(|e| {
                    CHApplyChangesError::AccountApplyChangesError(
                        CHAccountApplyChangesError::AccountShadowAllocsSumValueOnDiskInsertionError(
                            account_key.to_owned(),
                            *ephemeral_account_shadow_allocs_sum,
                            e,
                        ),
                    )
                })?;
            }
        }

        // 6. Save ephemeral shadow spaces.
        for (contract_id, ephemeral_shadow_space) in
            self.delta_contracts.updated_shadow_spaces.iter()
        {
            // 2.0 In-memory insertion.
            {
                // Get mutable in-memory permanent contract body.
                let in_memory_permanent_contract_body = self
                    .in_memory_contracts
                    .get_mut(contract_id)
                    .ok_or(CHApplyChangesError::ContractApplyChangesError(
                        CHContractApplyChangesError::UnableToGetContractBody(*contract_id),
                    ))?;

                // Update the shadow space in the in-memory permanent states.
                in_memory_permanent_contract_body.shadow_space = ephemeral_shadow_space.clone();
            }

            // 2.1 On-disk insertion.
            {
                // Open the contract tree using the contract ID as the tree name.
                let tree = self.on_disk_contracts.open_tree(contract_id).map_err(|e| {
                    CHApplyChangesError::ContractApplyChangesError(
                        CHContractApplyChangesError::OpenTreeError(*contract_id, e),
                    )
                })?;

                // Insert all shadows into the on-disk contract tree.
                for (ephemeral_shadow_account_key, ephemeral_shadow_alloc_value) in
                    ephemeral_shadow_space.allocs.iter()
                {
                    tree.insert(
                        ephemeral_shadow_account_key.to_vec(),
                        ephemeral_shadow_alloc_value.to_le_bytes().to_vec(),
                    )
                    .map_err(|e| {
                        CHApplyChangesError::ContractApplyChangesError(
                            CHContractApplyChangesError::ShadowAllocValueOnDiskInsertionError(
                                *contract_id,
                                *ephemeral_shadow_account_key,
                                *ephemeral_shadow_alloc_value,
                                e,
                            ),
                        )
                    })?;
                }

                // Also save the allocs sum with the special key (0xff..).
                tree.insert(
                    CONTRACT_ALLOCS_SUM_SPECIAL_KEY,
                    ephemeral_shadow_space.allocs_sum.to_le_bytes().to_vec(),
                )
                .map_err(|e| {
                    CHApplyChangesError::ContractApplyChangesError(
                        CHContractApplyChangesError::AllocsSumValueOnDiskInsertionError(
                            *contract_id,
                            ephemeral_shadow_space.allocs_sum,
                            e,
                        ),
                    )
                })?;
            }
        }

        // 7. Handle deallocations.
        {
            for (contract_id, ephemeral_dealloc_list) in self.delta_contracts.deallocs_list.iter() {
                // 3.0 In-memory deletion.
                {
                    // Get mutable in-memory permanent contract body.
                    let in_memory_permanent_contract_body = self
                        .in_memory_contracts
                        .get_mut(contract_id)
                        .ok_or(CHApplyChangesError::ContractApplyChangesError(
                            CHContractApplyChangesError::UnableToGetContractBody(*contract_id),
                        ))?;

                    // Remove all accounts from the shadow space.
                    for account_key in ephemeral_dealloc_list.iter() {
                        if in_memory_permanent_contract_body
                            .shadow_space
                            .allocs
                            .remove(account_key)
                            .is_none()
                        {
                            return Err(CHApplyChangesError::ContractApplyChangesError(
                                CHContractApplyChangesError::InMemoryDeallocAccountError(
                                    *contract_id,
                                    *account_key,
                                ),
                            ));
                        };
                    }
                }

                // 3.1 On-disk deletion.
                {
                    // Open the contract tree using the contract ID as the tree name.
                    let on_disk_permanent_shadow_space =
                        self.on_disk_contracts.open_tree(contract_id).map_err(|e| {
                            CHApplyChangesError::ContractApplyChangesError(
                                CHContractApplyChangesError::OpenTreeError(*contract_id, e),
                            )
                        })?;

                    // Remove all accounts from the shadow space.
                    for account_key in ephemeral_dealloc_list.iter() {
                        match on_disk_permanent_shadow_space.remove(account_key) {
                            Ok(_) => (),
                            Err(err) => {
                                return Err(CHApplyChangesError::ContractApplyChangesError(
                                    CHContractApplyChangesError::OnDiskDeallocAccountError(
                                        *contract_id,
                                        *account_key,
                                        err,
                                    ),
                                ));
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// Erase by db path.
pub fn erase_coin_holder(chain: Chain) {
    // Balance db path.
    let account_path = format!("db/{}/coin/account", chain.to_string());

    // Erase the path.
    let _ = std::fs::remove_dir_all(account_path);

    // Balance db path.
    let contract_path = format!("db/{}/coin/contract", chain.to_string());

    // Erase the path.
    let _ = std::fs::remove_dir_all(contract_path);
}
