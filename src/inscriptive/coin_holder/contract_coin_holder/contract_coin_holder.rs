use super::contract_coin_holder_error::{
    ContractCoinHolderApplyChangesError, ContractCoinHolderConstructionError,
};
use crate::inscriptive::coin_holder::contract_coin_holder::contract_coin_holder_error::{
    ContractBalanceDownError, ContractBalanceUpError, ContractCoinHolderRegisterContractError,
    ShadowAllocAccountError, ShadowAllocDownAllError, ShadowAllocDownError, ShadowAllocUpAllError,
    ShadowAllocUpError, ShadowDeallocAccountError,
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

/// Satoshi amount.
#[allow(non_camel_case_types)]
type SATOSHI_AMOUNT = u64;

/// A custom, high-precision satoshi amount.
/// 1 satoshi = 100,000,000 sati-satoshis.
#[allow(non_camel_case_types)]
type SATI_SATOSHI_AMOUNT = u128;

/// Special db key for the contract balance (0x00..).
const CONTRACT_BALANCE_SPECIAL_KEY: [u8; 32] = [0x00; 32];

/// Special db key for the allocs sum (0x01..).
const CONTRACT_ALLOCS_SUM_SPECIAL_KEY: [u8; 32] = [0x01; 32];

/// A struct for representing a shadow space of a contract.
#[derive(Clone)]
struct ShadowSpace {
    // Total allocated BTC value of the entire shadow space.
    pub allocs_sum: SATOSHI_AMOUNT,

    // Allocated BTC values of each account.
    pub allocs: HashMap<ACCOUNT_KEY, SATI_SATOSHI_AMOUNT>,
}

/// A struct for containing BTC balance and shadow space allocations of a contract.
#[derive(Clone)]
struct ContractBody {
    // Contract's BTC balance.
    pub balance: SATOSHI_AMOUNT,

    // Contract's shadow space.
    pub shadow_space: ShadowSpace,
}

/// A struct for containing state differences to be applied.
#[derive(Clone)]
struct Delta {
    // New contracts to register.
    pub new_contracts_to_register: Vec<CONTRACT_ID>,

    // New accounts to allocate for a given contract.
    pub allocs_list: HashMap<CONTRACT_ID, Vec<ACCOUNT_KEY>>,

    // Existing accounts to deallocate for a given contract.
    pub deallocs_list: HashMap<CONTRACT_ID, Vec<ACCOUNT_KEY>>,

    // Updated contract balances for a given contract.
    pub updated_contract_balances: HashMap<CONTRACT_ID, SATOSHI_AMOUNT>,

    // Updated shadow spaces for a given contract.
    pub updated_shadow_spaces: HashMap<CONTRACT_ID, ShadowSpace>,
}

/// A database manager for handling contract balances and shadow spaces.
///
/// NOTE: For now, we are caching *everything* in memory.
pub struct ContractCoinHolder {
    // IN-MEMORY STATES
    in_memory: HashMap<CONTRACT_ID, ContractBody>,

    // ON-DISK STATES
    on_disk: sled::Db,

    // STATE DIFFERENCES TO BE APPLIED
    delta: Delta,

    // BACKUP OF STATE DIFFERENCES IN CASE OF ROLLBACK
    delta_backup: Delta,
}

/// Guarded `ContractCoinHolder`.
#[allow(non_camel_case_types)]
pub type CONTRACT_COIN_HOLDER = Arc<Mutex<ContractCoinHolder>>;

// TODO: Implement a rank-based caching mechanism to only cache the high-ranked states.
// Right now, we are caching *ALL* contract states in memory.
impl ContractCoinHolder {
    /// Initialize the state for the given chain
    pub fn new(chain: Chain) -> Result<CONTRACT_COIN_HOLDER, ContractCoinHolderConstructionError> {
        // Open the respective database.
        let db_path = format!("db/{}/coin/contract", chain.to_string());
        let db = sled::open(db_path).map_err(ContractCoinHolderConstructionError::DBOpenError)?;

        // Initialize the in-memory list.
        let mut in_memory = HashMap::<CONTRACT_ID, ContractBody>::new();

        // Iterate over all trees in the database.
        for tree_name in db.tree_names() {
            // Deserialize contract id bytes from tree name.
            let contract_id: [u8; 32] = tree_name.as_ref().try_into().map_err(|_| {
                ContractCoinHolderConstructionError::UnableToDeserializeContractIDBytesFromTreeName(
                    tree_name.to_vec(),
                )
            })?;

            // Open the tree.
            let tree = db
                .open_tree(&tree_name)
                .map_err(|e| ContractCoinHolderConstructionError::TreeOpenError(contract_id, e))?;

            // Initialize the in-memory cache of shadow space allocations.
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
                        return Err(ContractCoinHolderConstructionError::TreeIterError(
                            contract_id,
                            index,
                            e,
                        ));
                    }
                };

                // Deserialize the key bytes.
                let tree_key_bytes: [u8; 32] = key.as_ref().try_into().map_err(|_| {
                    ContractCoinHolderConstructionError::UnableToDeserializeKeyBytesFromTreeKey(
                        contract_id,
                        index,
                        key.to_vec(),
                    )
                })?;

                // Match the tree key bytes.
                match tree_key_bytes {
                    // If the key is (0x00..), it is a special key that corresponds to the contract balance value.
                    CONTRACT_BALANCE_SPECIAL_KEY => {
                        // Deserialize the value bytes.
                        let contract_balance_value_in_satoshis: u64 =
                            u64::from_le_bytes(value.as_ref().try_into().map_err(|_| {
                                ContractCoinHolderConstructionError::UnableToDeserializeContractBalanceFromTreeValue(
                                    contract_id,
                                    index,
                                    tree_key_bytes,
                                    value.to_vec(),
                                )
                            })?);

                        // Update the contract balance.
                        contract_balance = contract_balance_value_in_satoshis;
                    }
                    // If the key is (0x01..), it is a special key that corresponds to the allocs sum value.
                    CONTRACT_ALLOCS_SUM_SPECIAL_KEY => {
                        // Deserialize the value bytes.
                        let allocs_sum_value_in_satoshis: u64 =
                            u64::from_le_bytes(value.as_ref().try_into().map_err(|_| {
                                ContractCoinHolderConstructionError::UnableToDeserializeAllocsSumFromTreeValue(
                                    contract_id,
                                    index,
                                    tree_key_bytes,
                                    value.to_vec(),
                                )
                            })?);

                        // Update the shadow space allocations sum.
                        allocs_sum = allocs_sum_value_in_satoshis;
                    }
                    _ => {
                        // This key is a normal account key that corresponds to an account allocation.

                        // Deserialize the value bytes.
                        let alloc_value_in_sati_satoshis: u128 =
                            u128::from_le_bytes(value.as_ref().try_into().map_err(|_| {
                                ContractCoinHolderConstructionError::UnableToDeserializeAllocValueFromTreeValue(
                                    contract_id,
                                    index,
                                    tree_key_bytes,
                                    value.to_vec(),
                                )
                            })?);

                        // Update the shadow space allocations.
                        allocs.insert(tree_key_bytes, alloc_value_in_sati_satoshis);
                    }
                }
            }

            // Check if the shadow space allocations sum exceeds the contract balance.
            if allocs_sum > contract_balance {
                return Err(
                    ContractCoinHolderConstructionError::AllocsSumExceedsTheContractBalance(
                        contract_id,
                        allocs_sum,
                        contract_balance,
                    ),
                );
            }

            // Create the contract body.
            let contract_body = ContractBody {
                balance: contract_balance,
                shadow_space: ShadowSpace { allocs_sum, allocs },
            };

            // Insert the contract body into the in-memory cache.
            in_memory.insert(contract_id, contract_body);
        }

        // Create a fresh new delta.
        let fresh_new_delta = Delta {
            new_contracts_to_register: Vec::new(),
            allocs_list: HashMap::new(),
            deallocs_list: HashMap::new(),
            updated_contract_balances: HashMap::new(),
            updated_shadow_spaces: HashMap::new(),
        };

        // Create the contract coin holder.
        let contract_coin_holder = ContractCoinHolder {
            in_memory,
            on_disk: db,
            delta: fresh_new_delta.clone(),
            delta_backup: fresh_new_delta,
        };

        // Create the guarded contract coin holder.
        let guarded_contract_coin_holder = Arc::new(Mutex::new(contract_coin_holder));

        // Return the guarded contract coin holder.
        Ok(guarded_contract_coin_holder)
    }

    /// Clones ephemeral states into the backup.
    fn backup_delta(&mut self) {
        self.delta_backup = self.delta.clone();
    }

    /// Restores ephemeral states from the backup.
    fn restore_delta(&mut self) {
        self.delta = self.delta_backup.clone();
    }

    /// Prepares the state holder prior to each execution.
    ///
    /// NOTE: Used by the Engine coordinator.
    pub fn pre_execution(&mut self) {
        // Backup the delta.
        self.backup_delta();
    }

    /// Checks if a contract is registered.
    ///
    /// NOTE: Permanant registrations only. Epheremal in-Delta registrations are out of scope.
    pub fn is_contract_registered(&self, contract_id: [u8; 32]) -> bool {
        self.in_memory.contains_key(&contract_id)
    }

    /// Get the contract coin balance for a contract ID.
    pub fn get_contract_balance(&self, contract_id: [u8; 32]) -> Option<u64> {
        // Try to get from the delta first.
        if let Some(balance) = self.delta.updated_contract_balances.get(&contract_id) {
            return Some(*balance);
        }

        // And then try to read from the in-memory states.
        self.in_memory.get(&contract_id).map(|body| body.balance)
    }

    /// Get the contract shadow allocation for a contract ID.
    pub fn get_contract_allocs_sum(&self, contract_id: [u8; 32]) -> Option<u64> {
        // Try to read from the delta first.
        if let Some(allocs_sum) = self.delta.updated_shadow_spaces.get(&contract_id) {
            return Some(allocs_sum.allocs_sum);
        }

        // And then try to get from the in-memory states.
        self.in_memory
            .get(&contract_id)
            .map(|body| body.shadow_space.allocs_sum)
    }

    /// Get the number of total shadow allocations of the contract.
    pub fn get_contract_num_allocs(&self, contract_id: [u8; 32]) -> Option<u64> {
        // Try to get from the delta first.
        if let Some(shadow_space) = self.delta.updated_shadow_spaces.get(&contract_id) {
            return Some(shadow_space.allocs.len() as u64);
        }

        // And then try to get from the in-memory states.
        self.in_memory
            .get(&contract_id)
            .map(|body| body.shadow_space.allocs.len() as u64)
    }

    /// Get the shadow allocation value of an account for a specific contract ID.
    pub fn get_account_shadow_alloc_value_in_sati_satoshis(
        &self,
        contract_id: [u8; 32],
        account_key: ACCOUNT_KEY,
    ) -> Option<u128> {
        // Check if the account is epheremally deallocated in the delta.
        if let Some(dealloc_list) = self.delta.deallocs_list.get(&contract_id) {
            if dealloc_list.contains(&account_key) {
                return None;
            }
        }

        // Try to read from the delta first.
        if let Some(shadow_space) = self.delta.updated_shadow_spaces.get(&contract_id) {
            return shadow_space.allocs.get(&account_key).cloned();
        }

        // And then try to read from the permanent states.
        self.in_memory
            .get(&contract_id)
            .and_then(|body| body.shadow_space.allocs.get(&account_key).cloned())
    }

    /// Get the shadow allocation value of an account for a specific contract ID in satoshis.
    pub fn get_account_shadow_alloc_value_in_satoshis(
        &self,
        contract_id: [u8; 32],
        account_key: ACCOUNT_KEY,
    ) -> Option<u64> {
        // Get the sati-satoshi value.
        let sati_satoshi_value =
            self.get_account_shadow_alloc_value_in_sati_satoshis(contract_id, account_key)?;

        // Divide by 100_000_000 to get the satoshi value.
        let satoshi_value = sati_satoshi_value / 100_000_000;

        // Return the result.
        Some(satoshi_value as u64)
    }

    /// Registers a contract if it is not already registered.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn register_contract(
        &mut self,
        contract_id: [u8; 32],
    ) -> Result<(), ContractCoinHolderRegisterContractError> {
        // Check if the contract has just been epheremally registered in the delta.
        if self.delta.new_contracts_to_register.contains(&contract_id) {
            return Err(
                ContractCoinHolderRegisterContractError::ContractHasJustBeenEphemerallyRegistered(
                    contract_id,
                ),
            );
        }

        // Check if the contract is already permanently registered.
        if self.in_memory.contains_key(&contract_id) {
            return Err(
                ContractCoinHolderRegisterContractError::ContractIsAlreadyPermanentlyRegistered(
                    contract_id,
                ),
            );
        }

        // Insert into the new contracts to register list in the delta.
        self.delta.new_contracts_to_register.push(contract_id);

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
    ) -> Result<(), ShadowAllocAccountError> {
        // Check if the account has just been epheremally allocated in the delta.
        // We do not allow it to be allocated again in the same execution.
        if let Some(allocs_list) = self.delta.allocs_list.get(&contract_id) {
            if allocs_list.contains(&account_key) {
                return Err(
                    ShadowAllocAccountError::AccountHasJustBeenEphemerallyAllocated(
                        contract_id,
                        account_key,
                    ),
                );
            }
        }

        // Check if the account has just been epheremally deallocated in the delta.
        // We do not allow it to be allocated after being deallocated in the same execution.
        if let Some(deallocs_list) = self.delta.deallocs_list.get(&contract_id) {
            if deallocs_list.contains(&account_key) {
                return Err(
                    ShadowAllocAccountError::AccountHasJustBeenEphemerallyDeallocated(
                        contract_id,
                        account_key,
                    ),
                );
            }
        }

        // Check if the account key is already permanently allocated by reading its allocation value.
        // We do not allow it to be allocated again if already permanently allocated.
        if self
            .get_account_shadow_alloc_value_in_sati_satoshis(contract_id, account_key)
            .is_some()
        {
            return Err(ShadowAllocAccountError::UnableToGetAccountAllocValue(
                contract_id,
                account_key,
            ));
        }

        // Retrieve the mutable shadow space from the delta.
        let epheremal_shadow_space = match self.delta.updated_shadow_spaces.get_mut(&contract_id) {
            Some(shadow_space) => shadow_space,
            None => {
                // Otherwise, from the permanent states.
                let contract_body = self.in_memory.get(&contract_id).ok_or(
                    ShadowAllocAccountError::UnableToGetContractBody(contract_id),
                )?;

                // Clone the shadow space from permanent states.
                let shadow_space = contract_body.shadow_space.clone();

                // Insert the shadow space into the delta.
                self.delta
                    .updated_shadow_spaces
                    .insert(contract_id, shadow_space);

                // Get the mutable shadow space from the delta that we just inserted.
                let delta_shadow_space = self
                    .delta
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
        self.delta
            .allocs_list
            .insert(contract_id, vec![account_key]);

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
    ) -> Result<(), ShadowDeallocAccountError> {
        // Check if the account has just been epheremally allocated in the delta.
        // We do not allow it to be deallocated if it is just allocated in the same execution.
        if self
            .delta
            .allocs_list
            .get(&contract_id)
            .unwrap_or(&Vec::new())
            .contains(&account_key)
        {
            return Err(
                ShadowDeallocAccountError::AccountHasJustBeenEphemerallyAllocated(
                    contract_id,
                    account_key,
                ),
            );
        }

        // Get the account's allocation value in sati-satoshis.
        // This also checks if the account is acutally permanently allocated.
        let allocation_value_in_sati_satoshis = self
            .get_account_shadow_alloc_value_in_sati_satoshis(contract_id, account_key)
            .ok_or(ShadowDeallocAccountError::UnableToGetAccountAllocValue(
                contract_id,
                account_key,
            ))?;

        // Check if the account allocation value is non-zero.
        // Deallocation is allowed only if the allocation value is zero.
        if allocation_value_in_sati_satoshis != 0 {
            return Err(ShadowDeallocAccountError::AllocValueIsNonZero(
                contract_id,
                account_key,
            ));
        }

        // Get the mutable epheremal dealloc list from the delta.
        let epheremal_dealloc_list = self.delta.deallocs_list.get_mut(&contract_id).ok_or(
            ShadowDeallocAccountError::UnableToGetEpheremalDeallocList(contract_id),
        )?;

        // Check if the account has just been epheremally deallocated in the delta.
        // We do not allow it to be deallocated if it is just deallocated in the same execution.
        if epheremal_dealloc_list.contains(&account_key) {
            return Err(
                ShadowDeallocAccountError::AccountHasJustBeenEphemerallyDeallocated(
                    contract_id,
                    account_key,
                ),
            );
        }

        // Insert the account key into the epheremal dealloc list.
        epheremal_dealloc_list.push(account_key);

        // Get the mutable epheremal shadow space from the delta.
        let epheremal_shadow_space = match self.delta.updated_shadow_spaces.get_mut(&contract_id) {
            Some(shadow_space) => shadow_space,
            None => {
                // Otherwise, from the permanent states.
                let contract_body = self.in_memory.get(&contract_id).ok_or(
                    ShadowDeallocAccountError::UnableToGetContractBody(contract_id),
                )?;

                // Clone the shadow space from permanent states.
                let shadow_space = contract_body.shadow_space.clone();

                // Insert the shadow space into the delta.
                self.delta
                    .updated_shadow_spaces
                    .insert(contract_id, shadow_space);

                // Get the mutable shadow space from the delta that we just inserted.
                let delta_shadow_space = self
                    .delta
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

    /// Increases contract's balance.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn contract_balance_up(
        &mut self,
        contract_id: [u8; 32],
        up_value_in_satoshis: u64,
    ) -> Result<(), ContractBalanceUpError> {
        // Get the old contract balance before any mutable borrows.
        let existing_contract_balance_in_satoshis: u64 =
            self.get_contract_balance(contract_id).ok_or(
                ContractBalanceUpError::UnableToGetContractBalance(contract_id),
            )?;

        // Calculate the new contract balance.
        let new_contract_balance_in_satoshis: u64 =
            existing_contract_balance_in_satoshis + up_value_in_satoshis;

        // Retrieve the mutable balance from the ephemeral states.
        let ephemeral_contract_balance =
            match self.delta.updated_contract_balances.get_mut(&contract_id) {
                // If the balance is already in the ephemeral states, return it.
                Some(balance) => balance,
                // Otherwise, from the permanent in-memory states.
                None => {
                    // Get the mutable balance from the permanent in-memory states.
                    let contract_body = self
                        .in_memory
                        .get_mut(&contract_id)
                        .ok_or(ContractBalanceUpError::UnableToGetContractBody(contract_id))?;

                    // Get the mutable balance.
                    let balance = contract_body.balance;

                    // Insert the balance into the ephemeral states.
                    self.delta
                        .updated_contract_balances
                        .insert(contract_id, balance);

                    // Get the mutable balance from the ephemeral that we just inserted.
                    let ephemeral_balance = self
                        .delta
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
    ) -> Result<(), ContractBalanceDownError> {
        // Get the old contract balance before any mutable borrows.
        let existing_contract_balance_in_satoshis: u64 =
            self.get_contract_balance(contract_id).ok_or(
                ContractBalanceDownError::UnableToGetContractBalance(contract_id),
            )?;

        // Check if the decrease would make the contract balance go below zero.
        if down_value_in_satoshis > existing_contract_balance_in_satoshis {
            return Err(ContractBalanceDownError::ContractBalanceWouldGoBelowZero(
                contract_id,
                existing_contract_balance_in_satoshis,
                down_value_in_satoshis,
            ));
        }

        // Calculate the new contract balance.
        let new_contract_balance_in_satoshis: u64 =
            existing_contract_balance_in_satoshis - down_value_in_satoshis;

        // Retrieve the mutable balance from the delta.
        let ephemeral_contract_balance =
            match self.delta.updated_contract_balances.get_mut(&contract_id) {
                // If the balance is already in the delta, return it.
                Some(balance) => balance,
                // Otherwise, from the permanent in-memory states.
                None => {
                    // Get the mutable balance from the permanent in-memory states.
                    let contract_body = self.in_memory.get_mut(&contract_id).ok_or(
                        ContractBalanceDownError::UnableToGetContractBody(contract_id),
                    )?;

                    // Get the mutable balance.
                    let balance = contract_body.balance;

                    // Insert the balance into the delta.
                    self.delta
                        .updated_contract_balances
                        .insert(contract_id, balance);

                    // Get the mutable balance from the delta that we just inserted.
                    let ephemeral_balance = self
                        .delta
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

    /// Increases an account's shadow allocation value in the contract's shadow space.    
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn shadow_up(
        &mut self,
        contract_id: [u8; 32],
        account_key: ACCOUNT_KEY,
        up_value_in_satoshis: u64,
    ) -> Result<(), ShadowAllocUpError> {
        // Convert the increase value to sati-satoshi value.
        let up_value_in_sati_satoshis: u128 = (up_value_in_satoshis as u128) * 100_000_000;

        // Get the old account allocation value and contract balance before any mutable borrows.
        let existing_account_shadow_alloc_value_in_sati_satoshis: u128 = self
            .get_account_shadow_alloc_value_in_sati_satoshis(contract_id, account_key)
            .ok_or(ShadowAllocUpError::UnableToGetAccountShadowAllocValue(
                contract_id,
                account_key,
            ))?;

        let existing_contract_balance_in_satoshis: u64 = self
            .get_contract_balance(contract_id)
            .ok_or(ShadowAllocUpError::UnableToGetContractBalance(contract_id))?;

        // Retrieve the mutable shadow space from the delta.
        let epheremal_shadow_space = match self.delta.updated_shadow_spaces.get_mut(&contract_id) {
            // If the shadow space is already in the ephemeral states, return it.
            Some(shadow_space) => shadow_space,
            // Otherwise, from the permanent in-memory states.
            None => {
                let contract_body = self
                    .in_memory
                    .get_mut(&contract_id)
                    .ok_or(ShadowAllocUpError::UnableToGetContractBody(contract_id))?;

                // Get the mutable shadow space.
                let shadow_space = contract_body.shadow_space.clone();

                // Insert the shadow space into the ephemeral states.
                self.delta
                    .updated_shadow_spaces
                    .insert(contract_id, shadow_space);

                // Get the mutable shadow space from the epheremal that we just inserted.
                let epheremal_shadow_space = self
                    .delta
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
            return Err(ShadowAllocUpError::AllocsSumExceedsTheContractBalance(
                contract_id,
                new_contract_allocs_sum_value_in_satoshis,
                existing_contract_balance_in_satoshis,
            ));
        }

        // Insert (or update) the account shadow allocation value into the ephemeral states.
        epheremal_shadow_space
            .allocs
            .insert(account_key, new_account_shadow_alloc_value_in_sati_satoshis);

        // Update the contract shadow allocation sum value.
        epheremal_shadow_space.allocs_sum = new_contract_allocs_sum_value_in_satoshis;

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
    ) -> Result<(), ShadowAllocDownError> {
        // Convert the decrease value to sati-satoshi value.
        let down_value_in_sati_satoshis: u128 = (down_value_in_satoshis as u128) * 100_000_000;

        // Get the old account allocation value and contract balance before any mutable borrows.
        let existing_account_shadow_alloc_value_in_sati_satoshis: u128 = self
            .get_account_shadow_alloc_value_in_sati_satoshis(contract_id, account_key)
            .ok_or(ShadowAllocDownError::UnableToGetAccountShadowAllocValue(
                contract_id,
                account_key,
            ))?;

        let existing_contract_balance_in_satoshis: u64 =
            self.get_contract_balance(contract_id).ok_or(
                ShadowAllocDownError::UnableToGetContractBalance(contract_id),
            )?;

        // Check if the decrease would make the allocation value go below zero.
        if down_value_in_sati_satoshis > existing_account_shadow_alloc_value_in_sati_satoshis {
            return Err(
                ShadowAllocDownError::AccountShadowAllocValueWouldGoBelowZero(
                    contract_id,
                    account_key,
                    existing_account_shadow_alloc_value_in_sati_satoshis,
                    down_value_in_sati_satoshis,
                ),
            );
        }

        // Retrieve the mutable shadow space from the delta.
        let epheremal_shadow_space = match self.delta.updated_shadow_spaces.get_mut(&contract_id) {
            // If the shadow space is already in the delta, return it.
            Some(shadow_space) => shadow_space,
            // Otherwise, from the permanent in-memory states.
            None => {
                let contract_body = self
                    .in_memory
                    .get_mut(&contract_id)
                    .ok_or(ShadowAllocDownError::UnableToGetContractBody(contract_id))?;

                // Get the mutable shadow space.
                let shadow_space = contract_body.shadow_space.clone();

                // Insert the shadow space into the delta.
                self.delta
                    .updated_shadow_spaces
                    .insert(contract_id, shadow_space);

                // Get the mutable shadow space from the epheremal that we just inserted.
                let epheremal_shadow_space = self
                    .delta
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
            return Err(ShadowAllocDownError::AllocsSumExceedsTheContractBalance(
                contract_id,
                new_contract_allocs_sum_value_in_satoshis,
                existing_contract_balance_in_satoshis,
            ));
        }

        // Insert (or update) the account shadow allocation value into the ephemeral states.
        epheremal_shadow_space
            .allocs
            .insert(account_key, new_account_shadow_alloc_value_in_sati_satoshis);

        // Update the contract shadow allocation sum value.
        epheremal_shadow_space.allocs_sum = new_contract_allocs_sum_value_in_satoshis;

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
    ) -> Result<u64, ShadowAllocUpAllError> {
        // Convert the increase value to sati-satoshi value.
        let up_value_in_sati_satoshis: u128 = (up_value_in_satoshis as u128) * 100_000_000;

        // Get the old contract balance and allocs sum before any mutable borrows.
        let contract_balance_in_satoshis: u64 = self.get_contract_balance(contract_id).ok_or(
            ShadowAllocUpAllError::UnableToGetContractBalance(contract_id),
        )?;

        // Get the old contract allocs sum before any mutable borrows.
        let existing_contract_allocs_sum_in_satoshis: u64 =
            self.get_contract_allocs_sum(contract_id).ok_or(
                ShadowAllocUpAllError::UnableToGetContractAllocsSum(contract_id),
            )?;

        // Check if the contract allocs sum is zero.
        // This operation is not possible with zero allocs sum.
        if existing_contract_allocs_sum_in_satoshis == 0 {
            return Err(ShadowAllocUpAllError::OperationNotPossibleWithZeroAllocsSum(contract_id));
        }

        // Calculate the new contract allocs sum value.
        let new_contract_allocs_sum_value_in_satoshis: u64 =
            existing_contract_allocs_sum_in_satoshis + up_value_in_satoshis;

        // Check if the new contract alloc sum value exceeds the contract balance.
        if new_contract_allocs_sum_value_in_satoshis > contract_balance_in_satoshis {
            return Err(ShadowAllocUpAllError::AllocsSumExceedsTheContractBalance(
                contract_id,
                new_contract_allocs_sum_value_in_satoshis,
                contract_balance_in_satoshis,
            ));
        }

        // Convert the old contract allocs sum to sati-satoshi value.
        let existing_contract_allocs_sum_in_satisatoshis: u128 =
            (existing_contract_allocs_sum_in_satoshis as u128) * 100_000_000;

        // Initialize a list of update values of individual accounts.
        let mut individual_update_values_in_sati_satoshis: HashMap<ACCOUNT_KEY, u128> =
            HashMap::new();

        // Iterate over all all account in the shadow space.
        for (account_key, shadow_alloc_value_in_sati_satoshis) in
            match self.delta.updated_shadow_spaces.get_mut(&contract_id) {
                // First try the ephemeral shadow space.
                Some(shadow_space) => shadow_space.allocs.iter(),
                // Otherwise from the in-memory shadow space.
                None => self
                    .in_memory
                    .get_mut(&contract_id)
                    .ok_or(ShadowAllocUpAllError::UnableToGetContractBody(contract_id))?
                    .shadow_space
                    .allocs
                    .iter(),
            }
        {
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
                individual_update_values_in_sati_satoshis
                    .insert(*account_key, individual_new_value_in_sati_satoshis);
            }
        }

        // Retrieve the mutable shadow space from the delta.
        let epheremal_shadow_space = match self.delta.updated_shadow_spaces.get_mut(&contract_id) {
            // If the shadow space is already in the delta, return it.
            Some(shadow_space) => shadow_space,
            // Otherwise, from the permanent in-memory states.
            None => {
                let contract_body = self
                    .in_memory
                    .get_mut(&contract_id)
                    .ok_or(ShadowAllocUpAllError::UnableToGetContractBody(contract_id))?;

                // Get the mutable shadow space.
                let shadow_space = contract_body.shadow_space.clone();

                // Insert the shadow space into the delta.
                self.delta
                    .updated_shadow_spaces
                    .insert(contract_id, shadow_space);

                // Get the mutable shadow space from the epheremal that we just inserted.
                let epheremal_shadow_space = self
                    .delta
                    .updated_shadow_spaces
                    .get_mut(&contract_id)
                    .expect("This cannot happen because we just inserted it.");

                // Return the shadow space.
                epheremal_shadow_space
            }
        };

        // Insert the individual up values into the ephemeral shadow space.
        for (account_key, individual_update_value_in_sati_satoshis) in
            individual_update_values_in_sati_satoshis.iter()
        {
            // Insert the new value into the ephemeral shadow space.
            epheremal_shadow_space
                .allocs
                .insert(*account_key, *individual_update_value_in_sati_satoshis);
        }

        // Update the allocs sum value in the ephemeral shadow space.
        epheremal_shadow_space.allocs_sum = new_contract_allocs_sum_value_in_satoshis;

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
    ) -> Result<u64, ShadowAllocDownAllError> {
        // Convert the decrease value to sati-satoshi value.
        let down_value_in_sati_satoshis: u128 = (down_value_in_satoshis as u128) * 100_000_000;

        // Get the old contract balance and allocs sum before any mutable borrows.
        let contract_balance_in_satoshis: u64 = self.get_contract_balance(contract_id).ok_or(
            ShadowAllocDownAllError::UnableToGetContractBalance(contract_id),
        )?;

        // Get the old contract allocs sum before any mutable borrows.
        let existing_contract_allocs_sum_in_satoshis: u64 =
            self.get_contract_allocs_sum(contract_id).ok_or(
                ShadowAllocDownAllError::UnableToGetContractAllocsSum(contract_id),
            )?;

        // Check if the contract allocs sum is zero.
        // This operation is not possible with zero allocs sum.
        if existing_contract_allocs_sum_in_satoshis == 0 {
            return Err(
                ShadowAllocDownAllError::OperationNotPossibleWithZeroAllocsSum(contract_id),
            );
        }

        // Check if would go below zero.
        if down_value_in_satoshis > existing_contract_allocs_sum_in_satoshis {
            return Err(ShadowAllocDownAllError::AllocsSumWouldGoBelowZero(
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
            return Err(ShadowAllocDownAllError::AllocsSumExceedsTheContractBalance(
                contract_id,
                new_contract_allocs_sum_value_in_satoshis,
                contract_balance_in_satoshis,
            ));
        }

        // Convert the old contract allocs sum to sati-satoshi value.
        let existing_contract_allocs_sum_in_satisatoshis: u128 =
            (existing_contract_allocs_sum_in_satoshis as u128) * 100_000_000;

        // Initialize a list of update values of individual accounts.
        let mut individual_update_values_in_sati_satoshis: HashMap<ACCOUNT_KEY, u128> =
            HashMap::new();

        // Iterate over all all account in the shadow space.
        for (account_key, shadow_alloc_value_in_sati_satoshis) in
            match self.delta.updated_shadow_spaces.get_mut(&contract_id) {
                // First try the ephemeral shadow space.
                Some(shadow_space) => shadow_space.allocs.iter(),
                // Otherwise from the in-memory shadow space.
                None => self
                    .in_memory
                    .get_mut(&contract_id)
                    .ok_or(ShadowAllocDownAllError::UnableToGetContractBody(
                        contract_id,
                    ))?
                    .shadow_space
                    .allocs
                    .iter(),
            }
        {
            // shadow_alloc_value_in_sati_satoshis divided by existing_contract_allocs_sum_in_satisatoshis = x divided by down_value_in_sati_satoshis.
            // NOTE: if the account is ephemerally deallocated, since it's allocation value had to be zero, this will also be zero.
            let individual_down_value_in_sati_satoshis: u128 = (shadow_alloc_value_in_sati_satoshis
                * down_value_in_sati_satoshis)
                / existing_contract_allocs_sum_in_satisatoshis;

            // Check if the individual down value would go below zero.
            if individual_down_value_in_sati_satoshis > *shadow_alloc_value_in_sati_satoshis {
                return Err(
                    ShadowAllocDownAllError::AccountShadowAllocValueWouldGoBelowZero(
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
                individual_update_values_in_sati_satoshis
                    .insert(*account_key, individual_new_value_in_sati_satoshis);
            }
        }

        // Retrieve the mutable shadow space from the delta.
        let epheremal_shadow_space = match self.delta.updated_shadow_spaces.get_mut(&contract_id) {
            // If the shadow space is already in the delta, return it.
            Some(shadow_space) => shadow_space,
            // Otherwise, from the permanent in-memory states.
            None => {
                let contract_body = self.in_memory.get_mut(&contract_id).ok_or(
                    ShadowAllocDownAllError::UnableToGetContractBody(contract_id),
                )?;

                // Get the mutable shadow space.
                let shadow_space = contract_body.shadow_space.clone();

                // Insert the shadow space into the delta.
                self.delta
                    .updated_shadow_spaces
                    .insert(contract_id, shadow_space);

                // Get the mutable shadow space from the epheremal that we just inserted.
                let epheremal_shadow_space = self
                    .delta
                    .updated_shadow_spaces
                    .get_mut(&contract_id)
                    .expect("This cannot happen because we just inserted it.");

                // Return the shadow space.
                epheremal_shadow_space
            }
        };

        // Insert the individual up values into the ephemeral shadow space.
        for (account_key, individual_update_value_in_sati_satoshis) in
            individual_update_values_in_sati_satoshis.iter()
        {
            // Insert the new value into the ephemeral shadow space.
            epheremal_shadow_space
                .allocs
                .insert(*account_key, *individual_update_value_in_sati_satoshis);
        }

        // Update the allocs sum value in the ephemeral shadow space.
        epheremal_shadow_space.allocs_sum = new_contract_allocs_sum_value_in_satoshis;

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
        self.delta.new_contracts_to_register.clear();
        self.delta.allocs_list.clear();
        self.delta.deallocs_list.clear();
        self.delta.updated_contract_balances.clear();
        self.delta.updated_shadow_spaces.clear();

        // Clear the ephemeral states backup.
        self.delta_backup.new_contracts_to_register.clear();
        self.delta_backup.allocs_list.clear();
        self.delta_backup.deallocs_list.clear();
        self.delta_backup.updated_contract_balances.clear();
        self.delta_backup.updated_shadow_spaces.clear();
    }

    /// Applies all epheremal changes from the delta into the in-memory and on-disk.
    pub fn apply_changes(&mut self) -> Result<(), ContractCoinHolderApplyChangesError> {
        // 0. Register new contracts.
        for contract_id in self.delta.new_contracts_to_register.iter() {
            // In-memory insertion.
            {
                let fresh_new_contract_body = ContractBody {
                    balance: 0,
                    shadow_space: ShadowSpace {
                        allocs_sum: 0,
                        allocs: HashMap::new(),
                    },
                };

                // Insert the contract body into the in-memory list.
                // Register the contract in-memory.
                self.in_memory.insert(*contract_id, fresh_new_contract_body);
            }

            // On-disk insertion.
            {
                // Open tree
                let tree = self.on_disk.open_tree(contract_id).map_err(|e| {
                    ContractCoinHolderApplyChangesError::OpenTreeError(*contract_id, e)
                })?;

                // Insert the contract body into the on-disk list.
                tree.insert(CONTRACT_BALANCE_SPECIAL_KEY, 0u64.to_le_bytes().to_vec())
                    .map_err(|e| {
                        ContractCoinHolderApplyChangesError::BalanceValueOnDiskInsertionError(
                            *contract_id,
                            0,
                            e,
                        )
                    })?;

                // Insert the shadow space into the on-disk list.
                tree.insert(CONTRACT_ALLOCS_SUM_SPECIAL_KEY, 0u64.to_le_bytes().to_vec())
                    .map_err(|e| {
                        ContractCoinHolderApplyChangesError::AllocsSumValueOnDiskInsertionError(
                            *contract_id,
                            0,
                            e,
                        )
                    })?;
            }
        }

        // 1. Save contract balances.
        for (contract_id, ephemeral_contract_balance) in self.delta.updated_contract_balances.iter()
        {
            // 1.0 In-memory insertion.
            {
                // Get mutable in-memory permanent contract body.
                let in_memory_permanent_contract_body = self.in_memory.get_mut(contract_id).ok_or(
                    ContractCoinHolderApplyChangesError::UnableToGetContractBody(*contract_id),
                )?;

                // Update the balance in the in-memory states.
                in_memory_permanent_contract_body.balance = *ephemeral_contract_balance;
            }

            // 1.1 On-disk insertion.
            {
                // Open tree
                let tree = self.on_disk.open_tree(contract_id).map_err(|e| {
                    ContractCoinHolderApplyChangesError::OpenTreeError(*contract_id, e)
                })?;

                // Save the balance to the balances db.
                tree.insert(
                    CONTRACT_BALANCE_SPECIAL_KEY,
                    ephemeral_contract_balance.to_le_bytes().to_vec(),
                )
                .map_err(|e| {
                    ContractCoinHolderApplyChangesError::BalanceValueOnDiskInsertionError(
                        *contract_id,
                        *ephemeral_contract_balance,
                        e,
                    )
                })?;
            }
        }

        // 2. Save ephemeral shadow spaces.
        for (contract_id, ephemeral_shadow_space) in self.delta.updated_shadow_spaces.iter() {
            // 2.0 In-memory insertion.
            {
                // Get mutable in-memory permanent contract body.
                let in_memory_permanent_contract_body = self.in_memory.get_mut(contract_id).ok_or(
                    ContractCoinHolderApplyChangesError::UnableToGetContractBody(*contract_id),
                )?;

                // Update the shadow space in the in-memory permanent states.
                in_memory_permanent_contract_body.shadow_space = ephemeral_shadow_space.clone();
            }

            // 2.1 On-disk insertion.
            {
                // Open the contract tree using the contract ID as the tree name.
                let tree = self.on_disk.open_tree(contract_id).map_err(|e| {
                    ContractCoinHolderApplyChangesError::OpenTreeError(*contract_id, e)
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
                        ContractCoinHolderApplyChangesError::ShadowAllocValueOnDiskInsertionError(
                            *contract_id,
                            *ephemeral_shadow_account_key,
                            *ephemeral_shadow_alloc_value,
                            e,
                        )
                    })?;
                }

                // Also save the allocs sum with the special key (0xff..).
                tree.insert(
                    CONTRACT_ALLOCS_SUM_SPECIAL_KEY,
                    ephemeral_shadow_space.allocs_sum.to_le_bytes().to_vec(),
                )
                .map_err(|e| {
                    ContractCoinHolderApplyChangesError::AllocsSumValueOnDiskInsertionError(
                        *contract_id,
                        ephemeral_shadow_space.allocs_sum,
                        e,
                    )
                })?;
            }
        }

        // 3. Handle deallocations.
        {
            for (contract_id, ephemeral_dealloc_list) in self.delta.deallocs_list.iter() {
                // 3.0 In-memory deletion.
                {
                    // Get mutable in-memory permanent contract body.
                    let in_memory_permanent_contract_body =
                        self.in_memory.get_mut(contract_id).ok_or(
                            ContractCoinHolderApplyChangesError::UnableToGetContractBody(
                                *contract_id,
                            ),
                        )?;

                    // Remove all accounts from the shadow space.
                    for account_key in ephemeral_dealloc_list.iter() {
                        if in_memory_permanent_contract_body
                            .shadow_space
                            .allocs
                            .remove(account_key)
                            .is_none()
                        {
                            return Err(
                                ContractCoinHolderApplyChangesError::InMemoryDeallocAccountError(
                                    *contract_id,
                                    *account_key,
                                ),
                            );
                        };
                    }
                }

                // 3.1 On-disk deletion.
                {
                    // Open the contract tree using the contract ID as the tree name.
                    let on_disk_permanent_shadow_space =
                        self.on_disk.open_tree(contract_id).map_err(|e| {
                            ContractCoinHolderApplyChangesError::OpenTreeError(*contract_id, e)
                        })?;

                    // Remove all accounts from the shadow space.
                    for account_key in ephemeral_dealloc_list.iter() {
                        match on_disk_permanent_shadow_space.remove(account_key) {
                            Ok(_) => (),
                            Err(err) => {
                                return Err(
                                    ContractCoinHolderApplyChangesError::OnDiskDeallocAccountError(
                                        *contract_id,
                                        *account_key,
                                        err,
                                    ),
                                );
                            }
                        }
                    }
                }
            }
        }

        // Clear the delta.
        self.flush_delta();

        Ok(())
    }
}

/// Erase by db path.
pub fn erase_contract_coin_holder(chain: Chain) {
    // Balance db path.
    let balance_path = format!("db/{}/coin/contract/balance", chain.to_string());

    // Shadow space db path.
    let shadow_space_path = format!("db/{}/coin/contract/shadow_space", chain.to_string());

    // Erase the paths.
    let _ = std::fs::remove_dir_all(balance_path);
    let _ = std::fs::remove_dir_all(shadow_space_path);
}
