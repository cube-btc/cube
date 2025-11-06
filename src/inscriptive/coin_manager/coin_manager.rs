use crate::inscriptive::coin_manager::bodies::account_body::account_body::CHAccountBody;
use crate::inscriptive::coin_manager::bodies::contract_body::contract_body::CHContractBody;
use crate::inscriptive::coin_manager::bodies::contract_body::shadow_space::shadow_space::ShadowSpace;
use crate::inscriptive::coin_manager::deltas::account_delta::account_delta::CHAccountDelta;
use crate::inscriptive::coin_manager::deltas::contract_delta::contract_delta::CHContractDelta;
use crate::inscriptive::coin_manager::errors::apply_changes_errors::{
    CMAccountApplyChangesError, CMApplyChangesError, CMContractApplyChangesError,
};
use crate::inscriptive::coin_manager::errors::balance_update_errors::{
    CMAccountBalanceDownError, CMAccountBalanceUpError, CMContractBalanceDownError,
    CMContractBalanceUpError,
};
use crate::inscriptive::coin_manager::errors::construction_errors::{
    CMConstructionAccountError, CMConstructionContractError, CMConstructionError,
};
use crate::inscriptive::coin_manager::errors::register_errors::{
    CMRegisterAccountError, CMRegisterContractError,
};
use crate::inscriptive::coin_manager::errors::shadow_alloc_errors::{
    CMContractShadowAllocAccountError, CMContractShadowDeallocAccountError,
};
use crate::inscriptive::coin_manager::errors::shadow_update_errors::{
    CMAccountShadowAllocsSumDownError, CMAccountShadowAllocsSumUpError, CMShadowDownAllError,
    CMShadowDownError, CMShadowUpAllError, CMShadowUpError,
};
use crate::operative::Chain;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Account key.
type AccountKey = [u8; 32];

/// Contract ID.
type ContractId = [u8; 32];

/// Sati-satoshi amount.
type SatiSatoshiAmount = u128;

/// One satoshi is 100_000_000 sati-satoshis.
const ONE_SATOSHI_IN_SATI_SATOSHIS: u128 = 100_000_000;

/// Special db key for the account balance (0x00..).
const ACCOUNT_BALANCE_SPECIAL_KEY: [u8; 1] = [0x00; 1];
/// Special db key for the account shadow allocs sum value (0x01..).
const ACCOUNT_ALLOCS_SUM_SPECIAL_KEY: [u8; 1] = [0x01; 1];

/// Special db key for the contract balance (0x00..).
const CONTRACT_BALANCE_SPECIAL_KEY: [u8; 32] = [0x00; 32];
/// Special db key for the contract shadow allocs sum value (0x01..).
const CONTRACT_ALLOCS_SUM_SPECIAL_KEY: [u8; 32] = [0x01; 32];

/// A database manager for handling account and contract balances & shadow space allocations.
pub struct CoinManager {
    // IN-MEMORY STATES
    in_memory_accounts: HashMap<AccountKey, CHAccountBody>,
    in_memory_contracts: HashMap<ContractId, CHContractBody>,

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

/// Guarded 'CoinManager'.
#[allow(non_camel_case_types)]
pub type COIN_MANAGER = Arc<Mutex<CoinManager>>;

impl CoinManager {
    pub fn new(chain: Chain) -> Result<COIN_MANAGER, CMConstructionError> {
        // 1. Open the account db.
        let account_db_path = format!("db/{}/coin/account", chain.to_string());
        let account_db = sled::open(account_db_path).map_err(|e| {
            CMConstructionError::AccountConstructionError(CMConstructionAccountError::DBOpenError(
                e,
            ))
        })?;

        // 2. Open the contract db.
        let contract_db_path = format!("db/{}/coin/contract", chain.to_string());
        let contract_db = sled::open(contract_db_path).map_err(|e| {
            CMConstructionError::ContractConstructionError(
                CMConstructionContractError::DBOpenError(e),
            )
        })?;

        // 3. Initialize the in-memory lists of account and contract bodies.
        let mut account_bodies = HashMap::<AccountKey, CHAccountBody>::new();
        let mut contract_bodies = HashMap::<ContractId, CHContractBody>::new();

        // 4. Collect account bodies from the account database.
        for tree_name in account_db.tree_names() {
            // 4.1. Deserialize account key bytes from tree name.
            let account_key: [u8; 32] = tree_name.as_ref().try_into().map_err(|_| {
                CMConstructionError::AccountConstructionError(
                    CMConstructionAccountError::UnableToDeserializeAccountKeyBytesFromTreeName(
                        tree_name.to_vec(),
                    ),
                )
            })?;

            // 4.2. Open the tree.
            let tree = account_db.open_tree(tree_name).map_err(|e| {
                CMConstructionError::AccountConstructionError(
                    CMConstructionAccountError::TreeOpenError(account_key, e),
                )
            })?;

            // 4.3. Initialize the account balance and shadow allocs sum.
            let mut account_balance: u64 = 0;
            let mut account_shadow_allocs_sum: u128 = 0;

            // 4.4. Iterate over all items in the tree.
            for (index, item) in tree.iter().enumerate() {
                // 4.4.1. Get the key and value.
                let (key, value) = match item {
                    Ok((k, v)) => (k, v),
                    Err(e) => {
                        return Err(CMConstructionError::AccountConstructionError(
                            CMConstructionAccountError::TreeIterError(index, e),
                        ));
                    }
                };

                // 4.4.2. Deserialize the key byte.
                let tree_key_byte: [u8; 1] = key.as_ref().try_into().map_err(|_| {
                    CMConstructionError::AccountConstructionError(
                        CMConstructionAccountError::UnableToDeserializeKeyBytesFromTreeKey(
                            account_key,
                            index,
                            key.to_vec(),
                        ),
                    )
                })?;

                // 4.4.3. Match the tree key bytes.
                match tree_key_byte {
                    // If the key is (0x00..), it is a special key that corresponds to the account balance value.
                    ACCOUNT_BALANCE_SPECIAL_KEY => {
                        // Deserialize the value bytes.
                        let account_balance_deserialized: u64 =
                            u64::from_le_bytes(value.as_ref().try_into().map_err(|_| {
                                CMConstructionError::AccountConstructionError(CMConstructionAccountError::UnableToDeserializeAccountBalanceFromTreeValue(
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
                                CMConstructionError::AccountConstructionError(CMConstructionAccountError::UnableToDeserializeAccountShadowAllocsSumFromTreeValue(
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
                        return Err(CMConstructionError::AccountConstructionError(
                            CMConstructionAccountError::InvalidTreeKeyEncountered(
                                account_key,
                                tree_key_byte.to_vec(),
                            ),
                        ));
                    }
                }

                
            }

            // Construct the account body.
            let account_body = CHAccountBody::new(account_balance, account_shadow_allocs_sum);

            // Insert the account body into the account bodies list.
            account_bodies.insert(account_key, account_body);
        }

        // 5. Collect contract bodies from the contract database.
        for tree_name in contract_db.tree_names() {
            // 5.1. Deserialize contract id bytes from tree name.
            let contract_id: [u8; 32] = tree_name.as_ref().try_into().map_err(|_| {
                CMConstructionError::ContractConstructionError(
                    CMConstructionContractError::UnableToDeserializeContractIDBytesFromTreeName(
                        tree_name.to_vec(),
                    ),
                )
            })?;

            // 5.2. Open the tree.
            let tree = contract_db.open_tree(&tree_name).map_err(|e| {
                CMConstructionError::ContractConstructionError(
                    CMConstructionContractError::TreeOpenError(contract_id, e),
                )
            })?;

            // 5.3. Initialize the list of shadow space allocations.
            let mut allocs = HashMap::<AccountKey, SatiSatoshiAmount>::new();

            // 5.4. Initialize the allocs sum and contract balance.
            let mut allocs_sum: u64 = 0;
            let mut contract_balance: u64 = 0;

            // 5.5. Iterate over all items in the tree.
            for (index, item) in tree.iter().enumerate() {
                // 5.5.1. Get the key and value.
                let (key, value) = match item {
                    Ok((k, v)) => (k, v),
                    Err(e) => {
                        return Err(CMConstructionError::ContractConstructionError(
                            CMConstructionContractError::TreeIterError(contract_id, index, e),
                        ));
                    }
                };

                // 5.5.2. Deserialize the key bytes.
                let tree_key_bytes: [u8; 32] = key.as_ref().try_into().map_err(|_| {
                    CMConstructionError::ContractConstructionError(
                        CMConstructionContractError::UnableToDeserializeKeyBytesFromTreeKey(
                            contract_id,
                            index,
                            key.to_vec(),
                        ),
                    )
                })?;

                // 5.5.3. Match the tree key bytes.
                match tree_key_bytes {
                    // If the key is (0x00..), it is a special key that corresponds to the contract balance value.
                    CONTRACT_BALANCE_SPECIAL_KEY => {
                        // Deserialize the value bytes.
                        let contract_balance_value_in_satoshis: u64 =
                                u64::from_le_bytes(value.as_ref().try_into().map_err(|_| {
                                    CMConstructionError::ContractConstructionError(CMConstructionContractError::UnableToDeserializeContractBalanceFromTreeValue(
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
                                    CMConstructionError::ContractConstructionError(CMConstructionContractError::UnableToDeserializeAllocsSumFromTreeValue(
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
                        // This key is an account key that corresponds to an allocation in the contract's shadow space.

                        // Deserialize the allocation value in sati-satoshis.
                        let alloc_value_in_sati_satoshis: u128 =
                                u128::from_le_bytes(value.as_ref().try_into().map_err(|_| {
                                    CMConstructionError::ContractConstructionError(CMConstructionContractError::UnableToDeserializeAllocValueFromTreeValue(
                                        contract_id,
                                        index,
                                        tree_key_bytes,
                                        value.to_vec(),
                                    ))
                                })?);

                        // Insert the allocation.
                        allocs.insert(tree_key_bytes, alloc_value_in_sati_satoshis);
                    }
                }
            }

            // 5.6. Check if the shadow space allocations sum exceeds the contract balance.
            if allocs_sum > contract_balance {
                return Err(CMConstructionError::ContractConstructionError(
                    CMConstructionContractError::AllocsSumExceedsTheContractBalance(
                        contract_id,
                        allocs_sum,
                        contract_balance,
                    ),
                ));
            }

            // 5.7. Construct the shadow space.
            let shadow_space = ShadowSpace::new(allocs_sum, allocs);

            // 5.8. Construct the contract body.
            let contract_body = CHContractBody::new(contract_balance, shadow_space);

            // 5.9. Insert the contract body into the contract bodies list.
            contract_bodies.insert(contract_id, contract_body);
        }

        // 6. Construct the coin holder.
        let coin_holder = CoinManager {
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

    /// Clones the deltas into the backup.   
    fn backup_delta(&mut self) {
        self.backup_of_delta_accounts = self.delta_accounts.clone();
        self.backup_of_delta_contracts = self.delta_contracts.clone();
    }

    /// Restores the deltas from the backup.
    fn restore_delta(&mut self) {
        self.delta_accounts = self.backup_of_delta_accounts.clone();
        self.delta_contracts = self.backup_of_delta_contracts.clone();
    }

    /// Returns the mutable ephemeral account balance from delta.
    fn get_mut_ephemeral_account_balance(&mut self, account_key: AccountKey) -> Option<&mut u64> {
        // If the account balance is not in the delta, create it.
        if !self
            .delta_accounts
            .updated_account_balances
            .contains_key(&account_key)
        {
            // Get the account body from the permanent in-memory states.
            let account_body = self.in_memory_accounts.get(&account_key)?;

            // Get the account balance.
            let balance = account_body.balance();

            // Insert the account balance into the delta.
            self.delta_accounts
                .updated_account_balances
                .insert(account_key, balance);
        }

        // Return the mutable ephemeral account balance.
        self.delta_accounts
            .updated_account_balances
            .get_mut(&account_key)
    }

    /// Returns the mutable ephemeral account shadow allocs sum value from delta.
    fn get_mut_ephemeral_account_shadow_allocs_sum(
        &mut self,
        account_key: AccountKey,
    ) -> Option<&mut u128> {
        // If the account shadow allocs sum is not in the delta, create it.
        if !self
            .delta_accounts
            .updated_shadow_allocs_sums
            .contains_key(&account_key)
        {
            // Get the account body from the permanent in-memory states.
            let account_body = self.in_memory_accounts.get(&account_key)?;

            // Get the account shadow allocs sum.
            let shadow_allocs_sum = account_body.shadow_allocs_sum();

            // Insert the account shadow allocs sum into the delta.
            self.delta_accounts
                .updated_shadow_allocs_sums
                .insert(account_key, shadow_allocs_sum);
        }

        // Return the mutable ephemeral account shadow allocs sum.
        self.delta_accounts
            .updated_shadow_allocs_sums
            .get_mut(&account_key)
    }

    /// Returns the mutable ephemeral allocs list from delta.
    fn get_mut_epheremal_contract_allocs_list(
        &mut self,
        contract_id: ContractId,
    ) -> Option<&mut Vec<AccountKey>> {
        // Check if the allocs list is in the delta.
        if !self.delta_contracts.allocs_list.contains_key(&contract_id) {
            // Insert a fresh allocs list into the delta.
            self.delta_contracts
                .allocs_list
                .insert(contract_id, Vec::new());
        }

        // Return the mutable ephemeral allocs list.
        self.delta_contracts.allocs_list.get_mut(&contract_id)
    }

    /// Returns the mutable ephemeral dealloc list from delta.
    fn get_mut_epheremal_contract_deallocs_list(
        &mut self,
        contract_id: ContractId,
    ) -> Option<&mut Vec<AccountKey>> {
        // Check if the dealloc list is in the delta.
        if !self
            .delta_contracts
            .deallocs_list
            .contains_key(&contract_id)
        {
            // Insert a fresh dealloc list into the delta.
            self.delta_contracts
                .deallocs_list
                .insert(contract_id, Vec::new());
        }

        // Return the mutable ephemeral dealloc list.
        self.delta_contracts.deallocs_list.get_mut(&contract_id)
    }

    /// Returns the mutable ephemeral contract balance from delta.
    fn get_mut_ephemeral_contract_balance(&mut self, contract_id: ContractId) -> Option<&mut u64> {
        // If the contract balance is not in the delta, create it.
        if !self
            .delta_contracts
            .updated_contract_balances
            .contains_key(&contract_id)
        {
            // Get the contract body from the permanent in-memory states.
            let contract_body = self.in_memory_contracts.get(&contract_id)?;

            // Get the contract balance.
            let balance = contract_body.balance();

            // Insert the contract balance into the delta.
            self.delta_contracts
                .updated_contract_balances
                .insert(contract_id, balance);
        }

        // Return the mutable ephemeral contract balance.
        self.delta_contracts
            .updated_contract_balances
            .get_mut(&contract_id)
    }

    /// Returns the mutable ephemeral shadow space from delta.
    fn get_mut_ephemeral_contract_shadow_space(
        &mut self,
        contract_id: ContractId,
    ) -> Option<&mut ShadowSpace> {
        // If the shadow space is not in the delta, create it.
        if !self
            .delta_contracts
            .updated_shadow_spaces
            .contains_key(&contract_id)
        {
            // Get the contract body from the permanent in-memory states.
            let contract_body = self.in_memory_contracts.get(&contract_id)?;

            // Get the shadow space.
            let shadow_space = contract_body.shadow_space().clone();

            // Insert the shadow space into the delta.
            self.delta_contracts
                .updated_shadow_spaces
                .insert(contract_id, shadow_space);
        }

        // Return the mutable ephemeral shadow space.
        self.delta_contracts
            .updated_shadow_spaces
            .get_mut(&contract_id)
    }

    /// Prepares 'CoinManager' prior to each execution.
    pub fn pre_execution(&mut self) {
        // Backup the deltas.
        self.backup_delta();
    }

    /// Returns the 'CHAccountBody' for a given account key.
    pub fn get_account_body(&self, account_key: AccountKey) -> Option<CHAccountBody> {
        self.in_memory_accounts.get(&account_key).cloned()
    }

    /// Returns the 'CHContractBody' for a given contract ID.
    pub fn get_contract_body(&self, contract_id: ContractId) -> Option<CHContractBody> {
        self.in_memory_contracts.get(&contract_id).cloned()
    }

    /// Checks if an account is permanently registered.
    ///
    /// NOTE: Does not check epheremal registrations in the delta.
    pub fn is_account_registered(&self, account_key: AccountKey) -> bool {
        self.in_memory_accounts.contains_key(&account_key)
    }

    /// Checks if a contract is permanently registered.
    ///
    /// NOTE: Does not check epheremal registrations in the delta.
    pub fn is_contract_registered(&self, contract_id: ContractId) -> bool {
        self.in_memory_contracts.contains_key(&contract_id)
    }

    /// Returns an account's balance in satoshis.
    pub fn get_account_balance(&self, account_key: AccountKey) -> Option<u64> {
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
            .map(|account_body| account_body.balance())
    }

    /// Returns a contract's balance in satoshis.
    pub fn get_contract_balance(&self, contract_id: ContractId) -> Option<u64> {
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
            .map(|contract_body| contract_body.balance())
    }

    /// Returns the sum of a given account's shadow allocation values across all contracts in sati-satoshis.
    pub fn get_account_shadow_allocs_sum_in_sati_satoshis(
        &self,
        account_key: AccountKey,
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
            .map(|account_body| account_body.shadow_allocs_sum())
    }

    /// Returns the sum of a given account's shadow allocation values across all contracts in satoshis.
    pub fn get_account_shadow_allocs_sum_in_satoshis(
        &self,
        account_key: AccountKey,
    ) -> Option<u64> {
        // Get the sati-satoshi value.
        let sati_satoshi_value =
            self.get_account_shadow_allocs_sum_in_sati_satoshis(account_key)?;

        // Convert to satoshi value.
        let satoshi_value = sati_satoshi_value / ONE_SATOSHI_IN_SATI_SATOSHIS;

        // Return the result.
        Some(satoshi_value as u64)
    }

    /// Returns the sum of all shadow allocation values of a given contract's shadow space in satoshis.
    pub fn get_contract_shadow_allocs_sum_in_satoshis(&self, contract_id: [u8; 32]) -> Option<u64> {
        // Try to read from the delta first.
        if let Some(allocs_sum) = self.delta_contracts.updated_shadow_spaces.get(&contract_id) {
            return Some(allocs_sum.allocs_sum());
        }

        // And then try to get from the in-memory states.
        self.in_memory_contracts
            .get(&contract_id)
            .map(|body| body.shadow_space().allocs_sum())
    }

    /// Returns the number of total shadow allocations of a given contract's shadow space.
    pub fn get_contract_num_shadow_allocs(&self, contract_id: [u8; 32]) -> Option<u64> {
        // Try to get from the delta first.
        if let Some(shadow_space) = self.delta_contracts.updated_shadow_spaces.get(&contract_id) {
            return Some(shadow_space.allocs_len() as u64);
        }

        // And then try to get from the in-memory states.
        self.in_memory_contracts
            .get(&contract_id)
            .map(|body| body.shadow_space().allocs_len() as u64)
    }

    /// Returns the shadow allocation value of a given account for a given contract in sati-satoshis.
    pub fn get_shadow_alloc_value_in_sati_satoshis(
        &self,
        contract_id: [u8; 32],
        account_key: AccountKey,
    ) -> Option<u128> {
        // Check if the account is epheremally deallocated in the delta.
        if let Some(dealloc_list) = self.delta_contracts.deallocs_list.get(&contract_id) {
            if dealloc_list.contains(&account_key) {
                // The account is epheremally deallocated in the same execution.
                // Therefore, there is no allocation value anymore to return.
                return None;
            }
        }

        // Try to read from the delta first.
        if let Some(shadow_space) = self.delta_contracts.updated_shadow_spaces.get(&contract_id) {
            return shadow_space.allocs().get(&account_key).cloned();
        }

        // And then try to read from the permanent states.
        self.in_memory_contracts
            .get(&contract_id)
            .and_then(|body| body.shadow_space().allocs().get(&account_key).cloned())
    }

    /// Returns the shadow allocation value of a given account for a given contract in satoshis.
    pub fn get_shadow_alloc_value_in_satoshis(
        &self,
        contract_id: [u8; 32],
        account_key: AccountKey,
    ) -> Option<u64> {
        // Get the sati-satoshi value.
        let sati_satoshi_value =
            self.get_shadow_alloc_value_in_sati_satoshis(contract_id, account_key)?;

        // Convert to satoshi value.
        let satoshi_value = sati_satoshi_value / ONE_SATOSHI_IN_SATI_SATOSHIS;

        // Return the result.
        Some(satoshi_value as u64)
    }

    /// Registers an account with the 'CoinManager'.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn register_account(
        &mut self,
        account_key: AccountKey,
        initial_account_balance: u64,
    ) -> Result<(), CMRegisterAccountError> {
        // Check if the account has just been epheremally registered in the delta.
        if self
            .delta_accounts
            .new_accounts_to_register
            .contains_key(&account_key)
        {
            return Err(
                CMRegisterAccountError::AccountHasJustBeenEphemerallyRegistered(account_key),
            );
        }

        // Check if the account is already permanently registered.
        if self.is_account_registered(account_key) {
            return Err(CMRegisterAccountError::AccountIsAlreadyPermanentlyRegistered(account_key));
        }

        // Insert into the new accounts to register list in the delta.
        self.delta_accounts
            .new_accounts_to_register
            .insert(account_key, initial_account_balance);

        // Return the result.
        Ok(())
    }

    /// Registers a contract with the 'CoinManager'.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn register_contract(
        &mut self,
        contract_id: [u8; 32],
        initial_contract_balance: u64,
    ) -> Result<(), CMRegisterContractError> {
        // Check if the contract has just been epheremally registered in the delta.
        if self
            .delta_contracts
            .new_contracts_to_register
            .contains_key(&contract_id)
        {
            return Err(
                CMRegisterContractError::ContractHasJustBeenEphemerallyRegistered(contract_id),
            );
        }

        // Check if the contract is already permanently registered.
        if self.is_contract_registered(contract_id) {
            return Err(
                CMRegisterContractError::ContractIsAlreadyPermanentlyRegistered(contract_id),
            );
        }

        // Insert into the new contracts to register list in the delta.
        self.delta_contracts
            .new_contracts_to_register
            .insert(contract_id, initial_contract_balance);

        // Return the result.
        Ok(())
    }

    /// Increases an account's balance.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn account_balance_up(
        &mut self,
        account_key: AccountKey,
        up_value_in_satoshis: u64,
    ) -> Result<(), CMAccountBalanceUpError> {
        // Get the account's existing balance.
        let account_balance_in_satoshis: u64 = self.get_account_balance(account_key).ok_or(
            CMAccountBalanceUpError::UnableToGetAccountBalance(account_key),
        )?;

        // Calculate the new account balance.
        let new_account_balance_in_satoshis: u64 =
            account_balance_in_satoshis + up_value_in_satoshis;

        // Get mutable ephemeral account balance from the delta.
        let mut_ephemeral_account_balance = self
            .get_mut_ephemeral_account_balance(account_key)
            .ok_or(CMAccountBalanceUpError::UnableToGetMutEphemeralAccountBalance(account_key))?;

        // Epheremally update the account's balance.
        *mut_ephemeral_account_balance = new_account_balance_in_satoshis;

        // Return the result.
        Ok(())
    }

    /// Decreases an account's balance.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn account_balance_down(
        &mut self,
        account_key: AccountKey,
        down_value_in_satoshis: u64,
    ) -> Result<(), CMAccountBalanceDownError> {
        // Get the account's existing balance.
        let account_balance_in_satoshis: u64 = self.get_account_balance(account_key).ok_or(
            CMAccountBalanceDownError::UnableToGetAccountBalance(account_key),
        )?;

        // Check if the decrease would make the account balance go below zero.
        if down_value_in_satoshis > account_balance_in_satoshis {
            return Err(CMAccountBalanceDownError::AccountBalanceWouldGoBelowZero(
                account_key,
                account_balance_in_satoshis,
                down_value_in_satoshis,
            ));
        }

        // Calculate the new account balance.
        let new_account_balance_in_satoshis: u64 =
            account_balance_in_satoshis - down_value_in_satoshis;

        // Get mutable ephemeral account balance from the delta.
        let mut_ephemeral_account_balance = self
            .get_mut_ephemeral_account_balance(account_key)
            .ok_or(CMAccountBalanceDownError::UnableToGetMutEphemeralAccountBalance(account_key))?;

        // Epheremally update the account's balance.
        *mut_ephemeral_account_balance = new_account_balance_in_satoshis;

        // Return the result.
        Ok(())
    }

    /// Increases a contract's balance.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn contract_balance_up(
        &mut self,
        contract_id: [u8; 32],
        up_value_in_satoshis: u64,
    ) -> Result<(), CMContractBalanceUpError> {
        // Get the contract's existing balance.
        let existing_contract_balance_in_satoshis: u64 =
            self.get_contract_balance(contract_id).ok_or(
                CMContractBalanceUpError::UnableToGetContractBalance(contract_id),
            )?;

        // Calculate the contract's new balance.
        let new_contract_balance_in_satoshis: u64 =
            existing_contract_balance_in_satoshis + up_value_in_satoshis;

        // Get mutable ephemeral contract balance from the delta.
        let mut_ephemeral_contract_balance =
            self.get_mut_ephemeral_contract_balance(contract_id).ok_or(
                CMContractBalanceUpError::UnableToGetMutEphemeralContractBalance(contract_id),
            )?;

        // Epheremally update the contract's balance.
        *mut_ephemeral_contract_balance = new_contract_balance_in_satoshis;

        // Return the result.
        Ok(())
    }

    /// Decreases a contract's balance.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn contract_balance_down(
        &mut self,
        contract_id: [u8; 32],
        down_value_in_satoshis: u64,
    ) -> Result<(), CMContractBalanceDownError> {
        // Get the contract's existing balance.
        let existing_contract_balance_in_satoshis: u64 =
            self.get_contract_balance(contract_id).ok_or(
                CMContractBalanceDownError::UnableToGetContractBalance(contract_id),
            )?;

        // Check if the decrease would make the contract balance go below zero.
        if down_value_in_satoshis > existing_contract_balance_in_satoshis {
            return Err(CMContractBalanceDownError::ContractBalanceWouldGoBelowZero(
                contract_id,
                existing_contract_balance_in_satoshis,
                down_value_in_satoshis,
            ));
        }

        // Calculate the contract's new balance.
        let new_contract_balance_in_satoshis: u64 =
            existing_contract_balance_in_satoshis - down_value_in_satoshis;

        // Get the contract's existing shadow allocs sum.
        let existing_contract_shadow_allocs_sum_in_satoshis: u64 = self
            .get_contract_shadow_allocs_sum_in_satoshis(contract_id)
            .ok_or(CMContractBalanceDownError::UnableToGetContractAllocsSum(
                contract_id,
            ))?;

        // Check if the contract balance would go below the allocs sum.
        // Shadow allocs sum is bound by the contract's balance.
        if new_contract_balance_in_satoshis < existing_contract_shadow_allocs_sum_in_satoshis {
            return Err(
                CMContractBalanceDownError::ContractBalanceWouldGoBelowAllocsSum(
                    contract_id,
                    new_contract_balance_in_satoshis,
                    existing_contract_shadow_allocs_sum_in_satoshis,
                ),
            );
        }

        // Get mutable ephemeral contract balance from the delta.
        let mut_ephemeral_contract_balance =
            self.get_mut_ephemeral_contract_balance(contract_id).ok_or(
                CMContractBalanceDownError::UnableToGetMutEphemeralContractBalance(contract_id),
            )?;

        // Epheremally update the contract's balance.
        *mut_ephemeral_contract_balance = new_contract_balance_in_satoshis;

        // Return the result.
        Ok(())
    }

    /// Allocates a new account in the contract's shadow space.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn contract_shadow_alloc_account(
        &mut self,
        contract_id: [u8; 32],
        account_key: AccountKey,
    ) -> Result<(), CMContractShadowAllocAccountError> {
        // 1. Check if the account has just been epheremally allocated in the delta.
        // We do not allow it to be allocated again in the same execution.
        if let Some(allocs_list) = self.delta_contracts.allocs_list.get(&contract_id) {
            if allocs_list.contains(&account_key) {
                return Err(
                    CMContractShadowAllocAccountError::AccountHasJustBeenEphemerallyAllocated(
                        contract_id,
                        account_key,
                    ),
                );
            }
        }

        // 2. Check if the account has just been epheremally deallocated in the delta.
        // We do not allow it to be allocated after being deallocated in the same execution.
        if let Some(deallocs_list) = self.delta_contracts.deallocs_list.get(&contract_id) {
            if deallocs_list.contains(&account_key) {
                return Err(
                    CMContractShadowAllocAccountError::AccountHasJustBeenEphemerallyDeallocated(
                        contract_id,
                        account_key,
                    ),
                );
            }
        }

        // 3. Check if the account key is already permanently allocated by reading its allocation value.
        // We do not allow it to be allocated again if already permanently allocated.
        if self
            .get_shadow_alloc_value_in_sati_satoshis(contract_id, account_key)
            .is_some()
        {
            return Err(
                CMContractShadowAllocAccountError::AccountIsAlreadyPermanentlyAllocated(
                    contract_id,
                    account_key,
                ),
            );
        }

        // 4. Epheremally insert the new allocation to the shadow space.
        {
            // 4.1. Get mutable ephemeral shadow space from the delta.
            let mut_epheremal_shadow_space = self
                .get_mut_ephemeral_contract_shadow_space(contract_id)
                .ok_or(
                    CMContractShadowAllocAccountError::UnableToGetMutEphemeralShadowSpace(
                        contract_id,
                    ),
                )?;

            // 4.2. Epheremally insert the new allocation with value initially set to zero.
            mut_epheremal_shadow_space.insert_update_alloc(account_key, 0);
        }

        // 5. Insert the allocation record to the allocs list.
        {
            // 5.1. Insert the allocation record to the allocs list in the delta.
            let mut_ephemeral_allocs_list = self
                .get_mut_epheremal_contract_allocs_list(contract_id)
                .ok_or(
                    CMContractShadowAllocAccountError::UnableToGetMutEpheremalAllocsList(
                        contract_id,
                    ),
                )?;

            // 5.2. Insert the allocation record to the allocs list.
            mut_ephemeral_allocs_list.push(account_key);
        }

        // 6. Return the result.
        Ok(())
    }

    /// Deallocates an account from the contract's shadow space.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn contract_shadow_dealloc_account(
        &mut self,
        contract_id: [u8; 32],
        account_key: AccountKey,
    ) -> Result<(), CMContractShadowDeallocAccountError> {
        // 1. Check if the account has just been epheremally allocated in the delta.
        // We do not allow it to be deallocated if it is just allocated in the same execution.
        if let Some(allocs_list) = self.delta_contracts.allocs_list.get(&contract_id) {
            if allocs_list.contains(&account_key) {
                return Err(
                    CMContractShadowDeallocAccountError::AccountHasJustBeenEphemerallyAllocated(
                        contract_id,
                        account_key,
                    ),
                );
            }
        }

        // 2. Check if the account has just been epheremally deallocated in the delta.
        if let Some(deallocs_list) = self.delta_contracts.deallocs_list.get(&contract_id) {
            if deallocs_list.contains(&account_key) {
                return Err(
                    CMContractShadowDeallocAccountError::AccountHasJustBeenEphemerallyDeallocated(
                        contract_id,
                        account_key,
                    ),
                );
            }
        }

        // 3. Get the account's allocation value in sati-satoshis.
        // This also checks if the account is acutally permanently allocated.
        let allocation_value_in_sati_satoshis = self
            .get_shadow_alloc_value_in_sati_satoshis(contract_id, account_key)
            .ok_or(
                CMContractShadowDeallocAccountError::UnableToGetAccountAllocValue(
                    contract_id,
                    account_key,
                ),
            )?;

        // 4. Check if the account allocation value is non-zero.
        // Deallocation is allowed only if the allocation value is zero.
        if allocation_value_in_sati_satoshis != 0 {
            return Err(CMContractShadowDeallocAccountError::AllocValueIsNonZero(
                contract_id,
                account_key,
            ));
        }

        // 5. Epheremally remove the account from the shadow space.
        {
            // 5.1. Get mutable ephemeral shadow space from the delta.
            let mut_epheremal_shadow_space = self
                .get_mut_ephemeral_contract_shadow_space(contract_id)
                .ok_or(
                    CMContractShadowDeallocAccountError::UnableToGetMutEphemeralShadowSpace(
                        contract_id,
                    ),
                )?;

            // 5.2. Epheremally remove the account key from the shadow space.
            mut_epheremal_shadow_space.remove_alloc(account_key);
        }

        // 6. Insert the deallocation record to the deallocs list.
        {
            // 6.1. Get the mutable epheremal dealloc list from the delta.
            let mut_epheremal_dealloc_list = self
                .get_mut_epheremal_contract_deallocs_list(contract_id)
                .ok_or(
                    CMContractShadowDeallocAccountError::UnableToGetMutEpheremalDeallocList(
                        contract_id,
                    ),
                )?;

            // 6.2. Insert the deallocation record to the deallocs list.
            mut_epheremal_dealloc_list.push(account_key);
        }

        // 7. Return the result.
        Ok(())
    }

    /// Increases an account's global shadow allocs sum value.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    fn account_shadow_allocs_sum_up(
        &mut self,
        account_key: AccountKey,
        up_value_in_sati_satoshis: u128,
    ) -> Result<(), CMAccountShadowAllocsSumUpError> {
        // 1. Get the existing account shadow allocs sum in sati-satoshis.
        let account_shadow_allocs_sum_in_sati_satoshis: u128 = self
            .get_account_shadow_allocs_sum_in_sati_satoshis(account_key)
            .ok_or(
                CMAccountShadowAllocsSumUpError::UnableToGetAccountShadowAllocsSum(account_key),
            )?;

        // 2. Calculate the new value.
        let new_account_shadow_allocs_sum_in_sati_satoshis: u128 =
            account_shadow_allocs_sum_in_sati_satoshis + up_value_in_sati_satoshis;

        // 3. Get mutable ephemeral account shadow allocs sum from the delta.
        let mut_ephemeral_account_shadow_allocs_sum = self
            .get_mut_ephemeral_account_shadow_allocs_sum(account_key)
            .ok_or(
                CMAccountShadowAllocsSumUpError::UnableToGetMutEphemeralAccountShadowAllocsSum(
                    account_key,
                ),
            )?;

        // 4. Epheremally update the account shadow allocs sum.
        *mut_ephemeral_account_shadow_allocs_sum = new_account_shadow_allocs_sum_in_sati_satoshis;

        // 5. Return the result.
        Ok(())
    }

    /// Decreases an account's global shadow allocs sum value.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    fn account_shadow_allocs_sum_down(
        &mut self,
        account_key: AccountKey,
        down_value_in_sati_satoshis: u128,
    ) -> Result<(), CMAccountShadowAllocsSumDownError> {
        // 1. Get the old ephemeral account shadow allocs sum before any mutable borrows.
        let account_shadow_allocs_sum_in_sati_satoshis: u128 = self
            .get_account_shadow_allocs_sum_in_sati_satoshis(account_key)
            .ok_or(
                CMAccountShadowAllocsSumDownError::UnableToGetAccountShadowAllocsSum(account_key),
            )?;

        // 2. Check if the decrease would make the account shadow allocs sum go below zero.
        if down_value_in_sati_satoshis > account_shadow_allocs_sum_in_sati_satoshis {
            return Err(
                CMAccountShadowAllocsSumDownError::AccountShadowAllocsSumWouldGoBelowZero(
                    account_key,
                    account_shadow_allocs_sum_in_sati_satoshis,
                    down_value_in_sati_satoshis,
                ),
            );
        }

        // 3. Calculate the new ephemeral account shadow allocs sum.
        let new_account_shadow_allocs_sum_in_sati_satoshis: u128 =
            account_shadow_allocs_sum_in_sati_satoshis - down_value_in_sati_satoshis;

        // 4. Get mutable ephemeral account shadow allocs sum from the delta.
        let mut_ephemeral_account_shadow_allocs_sum = self
            .get_mut_ephemeral_account_shadow_allocs_sum(account_key)
            .ok_or(
                CMAccountShadowAllocsSumDownError::UnableToGetMutEphemeralAccountShadowAllocsSum(
                    account_key,
                ),
            )?;

        // 5. Epheremally update the account shadow allocs sum.
        *mut_ephemeral_account_shadow_allocs_sum = new_account_shadow_allocs_sum_in_sati_satoshis;

        // 6. Return the result.
        Ok(())
    }

    /// Increases a given account's shadow allocation value in a given contract's shadow space.    
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn shadow_up(
        &mut self,
        contract_id: [u8; 32],
        account_key: AccountKey,
        up_value_in_satoshis: u64,
    ) -> Result<(), CMShadowUpError> {
        // 1. Convert the increase value to sati-satoshi value.
        let up_value_in_sati_satoshis: u128 =
            (up_value_in_satoshis as u128) * ONE_SATOSHI_IN_SATI_SATOSHIS;

        // 2. Get the account's existing shadow allocation value for this contract.
        let account_shadow_alloc_value_in_sati_satoshis: u128 = self
            .get_shadow_alloc_value_in_sati_satoshis(contract_id, account_key)
            .ok_or(CMShadowUpError::UnableToGetAccountShadowAllocValue(
                contract_id,
                account_key,
            ))?;

        // 3. Calculate the account's new shadow allocation value.
        let new_account_shadow_alloc_value_in_sati_satoshis: u128 =
            account_shadow_alloc_value_in_sati_satoshis + up_value_in_sati_satoshis;

        // 4. Get the contract's existing balance.
        let contract_balance_in_satoshis: u64 = self
            .get_contract_balance(contract_id)
            .ok_or(CMShadowUpError::UnableToGetContractBalance(contract_id))?;

        // 5. Get mutable ephemeral shadow space from the delta.
        let mut_epheremal_shadow_space = self
            .get_mut_ephemeral_contract_shadow_space(contract_id)
            .ok_or(CMShadowUpError::UnableToGetMutEphemeralShadowSpace(
                contract_id,
            ))?;

        // 6. Calculate the contract's new shadow allocs sum value.
        let new_contract_allocs_sum_value_in_satoshis: u64 =
            mut_epheremal_shadow_space.allocs_sum() + up_value_in_satoshis;

        // 7. Check if the contract's new shadow allocs sum value exceeds the contract balance.
        if new_contract_allocs_sum_value_in_satoshis > contract_balance_in_satoshis {
            return Err(CMShadowUpError::AllocsSumExceedsTheContractBalance(
                contract_id,
                new_contract_allocs_sum_value_in_satoshis,
                contract_balance_in_satoshis,
            ));
        }

        // 8. Epheremally update the account's shadow alloc value.
        mut_epheremal_shadow_space
            .insert_update_alloc(account_key, new_account_shadow_alloc_value_in_sati_satoshis);

        // 9. Epheremally update the contract's shadow allocs sum value.
        mut_epheremal_shadow_space.update_allocs_sum(new_contract_allocs_sum_value_in_satoshis);

        // 10. Update the account shadow allocs sum value.
        {
            self.account_shadow_allocs_sum_up(account_key, up_value_in_sati_satoshis)
                .map_err(|error| {
                    CMShadowUpError::AccountShadowAllocsSumUpError(contract_id, account_key, error)
                })?;
        }

        // 11. Return the result.
        Ok(())
    }

    /// Decreases a given account's shadow allocation value in a given contract's shadow space.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn shadow_down(
        &mut self,
        contract_id: [u8; 32],
        account_key: AccountKey,
        down_value_in_satoshis: u64,
    ) -> Result<(), CMShadowDownError> {
        // 1. Convert the decrease value to sati-satoshi value.
        let down_value_in_sati_satoshis: u128 =
            (down_value_in_satoshis as u128) * ONE_SATOSHI_IN_SATI_SATOSHIS;

        // 2. Get the account's existing shadow alloc value for this contract.
        let account_shadow_alloc_value_in_sati_satoshis: u128 = self
            .get_shadow_alloc_value_in_sati_satoshis(contract_id, account_key)
            .ok_or(CMShadowDownError::UnableToGetAccountShadowAllocValue(
                contract_id,
                account_key,
            ))?;

        // 3. Check if the decrease would make the account's alloc value to go below zero.
        if down_value_in_sati_satoshis > account_shadow_alloc_value_in_sati_satoshis {
            return Err(CMShadowDownError::AccountShadowAllocValueWouldGoBelowZero(
                contract_id,
                account_key,
                account_shadow_alloc_value_in_sati_satoshis,
                down_value_in_sati_satoshis,
            ));
        }

        // 4. Calculate the account's new shadow alloc value.
        let new_account_shadow_alloc_value_in_sati_satoshis: u128 =
            account_shadow_alloc_value_in_sati_satoshis - down_value_in_sati_satoshis;

        // 5. Get mutable ephemeral shadow space from the delta.
        let mut_epheremal_shadow_space = self
            .get_mut_ephemeral_contract_shadow_space(contract_id)
            .ok_or(CMShadowDownError::UnableToGetMutEphemeralShadowSpace(
                contract_id,
            ))?;

        // 5. Get the contract's existing shadow allocs sum value.
        let contract_shadow_allocs_sum_in_satoshis: u64 = mut_epheremal_shadow_space.allocs_sum();

        // 6. Check if the decrease would make the contract's shadow allocs sum to go below zero.
        // NOTE: This is unlikely to happen, but we are checking for it just in case.
        if down_value_in_satoshis > contract_shadow_allocs_sum_in_satoshis {
            return Err(CMShadowDownError::ContractShadowAllocsSumWouldGoBelowZero(
                contract_id,
                contract_shadow_allocs_sum_in_satoshis,
                down_value_in_satoshis,
            ));
        }

        // 7. Calculate the contract's new shadow allocs sum value.
        let new_contract_allocs_sum_value_in_satoshis: u64 =
            contract_shadow_allocs_sum_in_satoshis - down_value_in_satoshis;

        // 8. Epheremally update the account's shadow alloc value.
        mut_epheremal_shadow_space
            .insert_update_alloc(account_key, new_account_shadow_alloc_value_in_sati_satoshis);

        // 9. Epheremally update the contract's shadow allocs sum value.
        mut_epheremal_shadow_space.update_allocs_sum(new_contract_allocs_sum_value_in_satoshis);

        // 10. Epheremally update the account shadow allocs sum value.
        {
            self.account_shadow_allocs_sum_down(account_key, down_value_in_sati_satoshis)
                .map_err(|error| {
                    CMShadowDownError::AccountShadowAllocsSumDownError(
                        contract_id,
                        account_key,
                        error,
                    )
                })?;
        }

        // 11. Return the result.
        Ok(())
    }

    /// Proportionaly increases the shadow allocation value of all accounts in a contract shadow space by a given value.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn shadow_up_all(
        &mut self,
        contract_id: [u8; 32],
        up_value_in_satoshis: u64,
    ) -> Result<u64, CMShadowUpAllError> {
        // 1. Convert the increase value to sati-satoshi value.
        let up_value_in_sati_satoshis: u128 =
            (up_value_in_satoshis as u128) * ONE_SATOSHI_IN_SATI_SATOSHIS;

        // 2. Get the contract's existing balance.
        let contract_balance_in_satoshis: u64 = self
            .get_contract_balance(contract_id)
            .ok_or(CMShadowUpAllError::UnableToGetContractBalance(contract_id))?;

        // 3. Get the contract's existing shadow allocs sum value.
        let contract_shadow_allocs_sum_value_in_satoshis: u64 = self
            .get_contract_shadow_allocs_sum_in_satoshis(contract_id)
            .ok_or(CMShadowUpAllError::UnableToGetContractAllocsSum(
                contract_id,
            ))?;

        // 4. Check if the contract allocs sum is zero.
        // This operation is not possible with zero allocs sum.
        if contract_shadow_allocs_sum_value_in_satoshis == 0 {
            return Err(CMShadowUpAllError::OperationNotPossibleWithZeroAllocsSum(
                contract_id,
            ));
        }

        // 5. Calculate the new contract allocs sum value.
        let new_contract_allocs_sum_value_in_satoshis: u64 =
            contract_shadow_allocs_sum_value_in_satoshis + up_value_in_satoshis;

        // 6. Check if the new contract alloc sum value exceeds the contract balance.
        if new_contract_allocs_sum_value_in_satoshis > contract_balance_in_satoshis {
            return Err(CMShadowUpAllError::AllocsSumExceedsTheContractBalance(
                contract_id,
                new_contract_allocs_sum_value_in_satoshis,
                contract_balance_in_satoshis,
            ));
        }

        // 7. Convert the existing contract shadow allocs sum value to sati-satoshis.
        let contract_shadow_allocs_sum_value_in_sati_satoshis: u128 =
            (contract_shadow_allocs_sum_value_in_satoshis as u128) * ONE_SATOSHI_IN_SATI_SATOSHIS;

        // 8. Initialize a list of update values of individual accounts.
        // (up value, updated value)
        let mut individual_update_values_in_sati_satoshis: HashMap<AccountKey, (u128, u128)> =
            HashMap::new();

        // 9. Iterate over all all accounts in the shadow space.
        for (account_key, shadow_alloc_value_in_sati_satoshis) in match self
            .delta_contracts
            .updated_shadow_spaces
            .get(&contract_id)
        {
            // First try the ephemeral shadow space.
            Some(shadow_space) => shadow_space.allocs().iter(),
            // Otherwise from the in-memory shadow space.
            None => self
                .in_memory_contracts
                .get(&contract_id)
                .ok_or(CMShadowUpAllError::UnableToGetContractBody(contract_id))?
                .shadow_space()
                .allocs()
                .iter(),
        } {
            // shadow_alloc_value_in_sati_satoshis divided by existing_contract_allocs_sum_in_satisatoshis = x divided by up_value_in_sati_satoshis.
            // NOTE: if the account is ephemerally deallocated, since it's allocation value had to be zero, this will also be zero.
            let individual_up_value_in_sati_satoshis: u128 = (shadow_alloc_value_in_sati_satoshis
                * up_value_in_sati_satoshis)
                / contract_shadow_allocs_sum_value_in_sati_satoshis;

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

        // 10. Get the mutable shadow space from the delta.
        let mut_epheremal_shadow_space = self
            .get_mut_ephemeral_contract_shadow_space(contract_id)
            .ok_or(CMShadowUpAllError::UnableToGetMutEphemeralShadowSpace(
                contract_id,
            ))?;

        // 11. Epheremally update the account shadow allocation value.
        for (account_key, (_, individual_new_value_in_sati_satoshis)) in
            individual_update_values_in_sati_satoshis.iter()
        {
            mut_epheremal_shadow_space
                .insert_update_alloc(*account_key, *individual_new_value_in_sati_satoshis);
        }

        // 12. Epheremally update the contract's shadow allocs sum value.
        mut_epheremal_shadow_space.update_allocs_sum(new_contract_allocs_sum_value_in_satoshis);

        // 13. Epheremally update the account's shadow allocs sum value.
        for (account_key, (individual_up_value_in_sati_satoshis, _)) in
            individual_update_values_in_sati_satoshis.iter()
        {
            self.account_shadow_allocs_sum_up(*account_key, *individual_up_value_in_sati_satoshis)
                .map_err(|error| {
                    CMShadowUpAllError::AccountShadowAllocsSumUpError(
                        contract_id,
                        *account_key,
                        error,
                    )
                })?;
        }

        // 14. Return the number of affected accounts.
        Ok(individual_update_values_in_sati_satoshis.len() as u64)
    }

    /// Proportionaly decreases the shadow allocation value of all accounts in a contract shadow space by a given value.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn shadow_down_all(
        &mut self,
        contract_id: [u8; 32],
        down_value_in_satoshis: u64,
    ) -> Result<u64, CMShadowDownAllError> {
        // 1. Convert the decrease value to sati-satoshi value.
        let down_value_in_sati_satoshis: u128 =
            (down_value_in_satoshis as u128) * ONE_SATOSHI_IN_SATI_SATOSHIS;

        // 2. Get the old contract balance and allocs sum before any mutable borrows.
        let contract_balance_in_satoshis: u64 = self.get_contract_balance(contract_id).ok_or(
            CMShadowDownAllError::UnableToGetContractBalance(contract_id),
        )?;

        // 3. Get the old contract allocs sum before any mutable borrows.
        let existing_contract_allocs_sum_in_satoshis: u64 = self
            .get_contract_shadow_allocs_sum_in_satoshis(contract_id)
            .ok_or(CMShadowDownAllError::UnableToGetContractAllocsSum(
                contract_id,
            ))?;

        // 4. Check if the contract allocs sum is zero.
        // This operation is not possible with zero allocs sum.
        if existing_contract_allocs_sum_in_satoshis == 0 {
            return Err(CMShadowDownAllError::OperationNotPossibleWithZeroAllocsSum(
                contract_id,
            ));
        }

        // 5. Check if would go below zero.
        if down_value_in_satoshis > existing_contract_allocs_sum_in_satoshis {
            return Err(CMShadowDownAllError::AllocsSumWouldGoBelowZero(
                contract_id,
                existing_contract_allocs_sum_in_satoshis,
                down_value_in_satoshis,
            ));
        }

        // 6. Calculate the new contract allocs sum value.
        let new_contract_allocs_sum_value_in_satoshis: u64 =
            existing_contract_allocs_sum_in_satoshis - down_value_in_satoshis;

        // 7. Check if the new contract alloc sum value exceeds the contract balance.
        if new_contract_allocs_sum_value_in_satoshis > contract_balance_in_satoshis {
            return Err(CMShadowDownAllError::AllocsSumExceedsTheContractBalance(
                contract_id,
                new_contract_allocs_sum_value_in_satoshis,
                contract_balance_in_satoshis,
            ));
        }

        // 8. Convert the old contract allocs sum to sati-satoshi value.
        let existing_contract_allocs_sum_in_satisatoshis: u128 =
            (existing_contract_allocs_sum_in_satoshis as u128) * ONE_SATOSHI_IN_SATI_SATOSHIS;

        // 9. Initialize a list of update values of individual accounts.
        // (down value, updated value)
        let mut individual_update_values_in_sati_satoshis: HashMap<AccountKey, (u128, u128)> =
            HashMap::new();

        // 10. Iterate over all all accounts in the shadow space.
        for (account_key, shadow_alloc_value_in_sati_satoshis) in match self
            .delta_contracts
            .updated_shadow_spaces
            .get(&contract_id)
        {
            // First try the ephemeral shadow space.
            Some(shadow_space) => shadow_space.allocs().iter(),
            // Otherwise from the in-memory shadow space.
            None => self
                .in_memory_contracts
                .get(&contract_id)
                .ok_or(CMShadowDownAllError::UnableToGetContractBody(contract_id))?
                .shadow_space()
                .allocs()
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
                    CMShadowDownAllError::AccountShadowAllocValueWouldGoBelowZero(
                        contract_id,
                        *account_key,
                        *shadow_alloc_value_in_sati_satoshis,
                        individual_down_value_in_sati_satoshis,
                    ),
                );
            }

            // If the individual down value is greater than zero, insert it into the list of new values.
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

        // 11. Retrieve the mutable shadow space from the delta.
        let mut_epheremal_shadow_space = self
            .get_mut_ephemeral_contract_shadow_space(contract_id)
            .ok_or(CMShadowDownAllError::UnableToGetMutEphemeralShadowSpace(
                contract_id,
            ))?;

        // 12. Epheremally update the account shadow allocation value.
        for (account_key, (_, individual_new_value_in_sati_satoshis)) in
            individual_update_values_in_sati_satoshis.iter()
        {
            mut_epheremal_shadow_space
                .insert_update_alloc(*account_key, *individual_new_value_in_sati_satoshis);
        }

        // 13. Epheremally update the allocs sum value in the shadow space.
        mut_epheremal_shadow_space.update_allocs_sum(new_contract_allocs_sum_value_in_satoshis);

        // 14. Epheremally update the account shadow allocs sum value.
        for (account_key, (individual_down_value_in_sati_satoshis, _)) in
            individual_update_values_in_sati_satoshis.iter()
        {
            self.account_shadow_allocs_sum_down(
                *account_key,
                *individual_down_value_in_sati_satoshis,
            )
            .map_err(|error| {
                CMShadowDownAllError::AccountShadowAllocsSumDownError(
                    contract_id,
                    *account_key,
                    error,
                )
            })?;
        }

        // 15. Return the number of affected accounts.
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

    /// Applies all epheremal changes from the delta into the permanent in-memory & on-disk.
    pub fn apply_changes(&mut self) -> Result<(), CMApplyChangesError> {
        // 1. Register new accounts in-memory and on-disk.
        for (account_key, initial_account_balance) in
            self.delta_accounts.new_accounts_to_register.iter()
        {
            // A fresh new account has a zero allocs sum value.
            let initial_account_allocs_sum_value_in_sati_satoshis: u128 = 0;

            // 1.1 In-memory insertion.
            {
                // Construct the fresh new account body.
                let fresh_new_account_body = CHAccountBody::new(
                    *initial_account_balance,
                    initial_account_allocs_sum_value_in_sati_satoshis,
                );

                // Insert the account balance into the in-memory list.
                // Register the account in-memory with zero balance.
                self.in_memory_accounts
                    .insert(*account_key, fresh_new_account_body);
            }

            // 1.2 On-disk insertion.
            {
                // Open on-disk accounts tree.
                let tree = self.on_disk_accounts.open_tree(account_key).map_err(|e| {
                    CMApplyChangesError::AccountApplyChangesError(
                        CMAccountApplyChangesError::OpenTreeError(*account_key, e),
                    )
                })?;

                // Insert the account balance on-disk.
                {
                    tree.insert(
                        ACCOUNT_BALANCE_SPECIAL_KEY,
                        initial_account_balance.to_le_bytes().to_vec(),
                    )
                    .map_err(|e| {
                        CMApplyChangesError::AccountApplyChangesError(
                            CMAccountApplyChangesError::BalanceValueOnDiskInsertionError(
                                account_key.to_owned(),
                                *initial_account_balance,
                                e,
                            ),
                        )
                    })?;
                }

                // Insert the account shadow allocs value sum on-disk.
                {
                    tree.insert(
                        ACCOUNT_ALLOCS_SUM_SPECIAL_KEY,
                        initial_account_allocs_sum_value_in_sati_satoshis
                            .to_le_bytes()
                            .to_vec(),
                    )
                    .map_err(|e| {
                        CMApplyChangesError::AccountApplyChangesError(
                            CMAccountApplyChangesError::ShadowAllocsSumValueOnDiskInsertionError(
                                *account_key,
                                initial_account_allocs_sum_value_in_sati_satoshis,
                                e,
                            ),
                        )
                    })?;
                }
            }
        }

        // 2. Register new contracts in-memory and on-disk.
        for (contract_id, initial_contract_balance) in
            self.delta_contracts.new_contracts_to_register.iter()
        {
            // A fresh new contract has a zero allocs sum value.
            let initial_contract_allocs_sum_value_in_satoshis: u64 = 0;

            // 2.1 In-memory insertion.
            {
                // Construct the fresh new shadow space.
                let fresh_new_shadow_space = ShadowSpace::fresh_new();

                // Construct the fresh new contract body.
                let fresh_new_contract_body =
                    CHContractBody::new(*initial_contract_balance, fresh_new_shadow_space);

                // Insert the contract body into the in-memory list.
                // Register the contract in-memory.
                self.in_memory_contracts
                    .insert(*contract_id, fresh_new_contract_body);
            }

            // 2.2 On-disk insertion.
            {
                // Open tree
                let tree = self.on_disk_contracts.open_tree(contract_id).map_err(|e| {
                    CMApplyChangesError::ContractApplyChangesError(
                        CMContractApplyChangesError::OpenTreeError(*contract_id, e),
                    )
                })?;

                // Insert the contract balance on-disk.
                tree.insert(
                    CONTRACT_BALANCE_SPECIAL_KEY,
                    initial_contract_balance.to_le_bytes().to_vec(),
                )
                .map_err(|e| {
                    CMApplyChangesError::ContractApplyChangesError(
                        CMContractApplyChangesError::BalanceValueOnDiskInsertionError(
                            *contract_id,
                            *initial_contract_balance,
                            e,
                        ),
                    )
                })?;

                // Insert the contract allocs sum value on-disk.
                tree.insert(
                    CONTRACT_ALLOCS_SUM_SPECIAL_KEY,
                    initial_contract_allocs_sum_value_in_satoshis
                        .to_le_bytes()
                        .to_vec(),
                )
                .map_err(|e| {
                    CMApplyChangesError::ContractApplyChangesError(
                        CMContractApplyChangesError::AllocsSumValueOnDiskInsertionError(
                            *contract_id,
                            initial_contract_allocs_sum_value_in_satoshis,
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
            // 3.1 In-memory insertion.
            {
                // Get the mutable permanent account body from the permanent states.
                let mut_permanent_account_body = self
                    .in_memory_accounts
                    .get_mut(account_key)
                    .ok_or(CMApplyChangesError::AccountApplyChangesError(
                        CMAccountApplyChangesError::UnableToGetPermanentAccountBody(*account_key),
                    ))?;

                // Update the account balance in-memory.
                mut_permanent_account_body.update_balance(*ephemeral_account_balance);
            }

            // 3.2 On-disk insertion.
            {
                // Open tree.
                let tree = self.on_disk_accounts.open_tree(account_key).map_err(|e| {
                    CMApplyChangesError::AccountApplyChangesError(
                        CMAccountApplyChangesError::OpenTreeError(*account_key, e),
                    )
                })?;

                // Update the account balance on-disk.
                tree.insert(
                    ACCOUNT_BALANCE_SPECIAL_KEY,
                    ephemeral_account_balance.to_le_bytes().to_vec(),
                )
                .map_err(|e| {
                    CMApplyChangesError::AccountApplyChangesError(
                        CMAccountApplyChangesError::BalanceValueOnDiskInsertionError(
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
            // 4.1 In-memory insertion.
            {
                // Get mutable permanent contract body.
                let mut_permanent_contract_body = self
                    .in_memory_contracts
                    .get_mut(contract_id)
                    .ok_or(CMApplyChangesError::ContractApplyChangesError(
                        CMContractApplyChangesError::UnableToGetPermanentContractBody(*contract_id),
                    ))?;

                // Update the contract balance in-memory.
                mut_permanent_contract_body.update_balance(*ephemeral_contract_balance);
            }

            // 4.2 On-disk insertion.
            {
                // Open tree.
                let tree = self.on_disk_contracts.open_tree(contract_id).map_err(|e| {
                    CMApplyChangesError::ContractApplyChangesError(
                        CMContractApplyChangesError::OpenTreeError(*contract_id, e),
                    )
                })?;

                // Update the contract balance on-disk.
                tree.insert(
                    CONTRACT_BALANCE_SPECIAL_KEY,
                    ephemeral_contract_balance.to_le_bytes().to_vec(),
                )
                .map_err(|e| {
                    CMApplyChangesError::ContractApplyChangesError(
                        CMContractApplyChangesError::BalanceValueOnDiskInsertionError(
                            *contract_id,
                            *ephemeral_contract_balance,
                            e,
                        ),
                    )
                })?;
            }
        }

        // 5. Save account's updated shadow allocs sum values.
        // NOTE: This also automatically handles new allocations.
        for (account_key, ephemeral_account_shadow_allocs_sum) in
            self.delta_accounts.updated_shadow_allocs_sums.iter()
        {
            // 5.1 In-memory insertion.
            {
                // Get the mutable permanent account body.
                let mut_permanent_account_body = self
                    .in_memory_accounts
                    .get_mut(account_key)
                    .ok_or(CMApplyChangesError::AccountApplyChangesError(
                        CMAccountApplyChangesError::UnableToGetPermanentAccountBody(*account_key),
                    ))?;

                // Update the shadow allocs sum in-memory.
                mut_permanent_account_body
                    .update_shadow_allocs_sum(*ephemeral_account_shadow_allocs_sum);
            }

            // 5.2 On-disk insertion.
            {
                // Open tree.
                let tree = self.on_disk_accounts.open_tree(account_key).map_err(|e| {
                    CMApplyChangesError::AccountApplyChangesError(
                        CMAccountApplyChangesError::OpenTreeError(*account_key, e),
                    )
                })?;

                // Update the shadow allocs sum on-disk.
                tree.insert(
                    ACCOUNT_ALLOCS_SUM_SPECIAL_KEY,
                    ephemeral_account_shadow_allocs_sum.to_le_bytes().to_vec(),
                )
                .map_err(|e| {
                    CMApplyChangesError::AccountApplyChangesError(
                        CMAccountApplyChangesError::ShadowAllocsSumValueOnDiskInsertionError(
                            account_key.to_owned(),
                            *ephemeral_account_shadow_allocs_sum,
                            e,
                        ),
                    )
                })?;
            }
        }

        // 6. Save contract's updated shadow spaces.
        for (contract_id, ephemeral_shadow_space) in
            self.delta_contracts.updated_shadow_spaces.iter()
        {
            // Get the contract's ephemeral shadow allocs sum value.
            let epheremal_shadow_allocs_sum_value = ephemeral_shadow_space.allocs_sum();

            // 6.1 In-memory insertion.
            {
                // Get mutable permanent contract body.
                let mut_permanent_contract_body = self
                    .in_memory_contracts
                    .get_mut(contract_id)
                    .ok_or(CMApplyChangesError::ContractApplyChangesError(
                        CMContractApplyChangesError::UnableToGetPermanentContractBody(*contract_id),
                    ))?;

                // Update the shadow space in-memory.
                mut_permanent_contract_body.update_shadow_space(ephemeral_shadow_space.clone());
            }

            // 6.2 On-disk insertion.
            {
                // Open tree.
                let tree = self.on_disk_contracts.open_tree(contract_id).map_err(|e| {
                    CMApplyChangesError::ContractApplyChangesError(
                        CMContractApplyChangesError::OpenTreeError(*contract_id, e),
                    )
                })?;

                // Update alloc values one-by-one on-disk.
                for (shadow_account_key, ephemeral_shadow_alloc_value) in
                    ephemeral_shadow_space.allocs().iter()
                {
                    // Update the shadow alloc value on-disk.
                    tree.insert(
                        shadow_account_key.to_vec(),
                        ephemeral_shadow_alloc_value.to_le_bytes().to_vec(),
                    )
                    .map_err(|e| {
                        CMApplyChangesError::ContractApplyChangesError(
                            CMContractApplyChangesError::ShadowAllocValueOnDiskInsertionError(
                                *contract_id,
                                *shadow_account_key,
                                *ephemeral_shadow_alloc_value,
                                e,
                            ),
                        )
                    })?;
                }

                // Update the allocs sum value on-disk.
                tree.insert(
                    CONTRACT_ALLOCS_SUM_SPECIAL_KEY,
                    epheremal_shadow_allocs_sum_value.to_le_bytes().to_vec(),
                )
                .map_err(|e| {
                    CMApplyChangesError::ContractApplyChangesError(
                        CMContractApplyChangesError::AllocsSumValueOnDiskInsertionError(
                            *contract_id,
                            epheremal_shadow_allocs_sum_value,
                            e,
                        ),
                    )
                })?;
            }
        }

        // 7. Handle deallocations.
        {
            for (contract_id, ephemeral_dealloc_list) in self.delta_contracts.deallocs_list.iter() {
                // 7.1 In-memory deletion.
                {
                    // Get mutable permanent contract body.
                    let mut_permanent_contract_body = self
                        .in_memory_contracts
                        .get_mut(contract_id)
                        .ok_or(CMApplyChangesError::ContractApplyChangesError(
                            CMContractApplyChangesError::UnableToGetPermanentContractBody(
                                *contract_id,
                            ),
                        ))?;

                    // Remove all accounts from the shadow space.
                    for account_key in ephemeral_dealloc_list.iter() {
                        if !mut_permanent_contract_body
                            .shadow_space_mut()
                            .remove_alloc(*account_key)
                        {
                            return Err(CMApplyChangesError::ContractApplyChangesError(
                                CMContractApplyChangesError::InMemoryDeallocAccountError(
                                    *contract_id,
                                    *account_key,
                                ),
                            ));
                        };
                    }
                }

                // 7.2 On-disk deletion.
                {
                    // Open tree.
                    let tree = self.on_disk_contracts.open_tree(contract_id).map_err(|e| {
                        CMApplyChangesError::ContractApplyChangesError(
                            CMContractApplyChangesError::OpenTreeError(*contract_id, e),
                        )
                    })?;

                    // Remove all accounts from the shadow space.
                    for account_key in ephemeral_dealloc_list.iter() {
                        match tree.remove(account_key) {
                            Ok(_) => (),
                            Err(err) => {
                                return Err(CMApplyChangesError::ContractApplyChangesError(
                                    CMContractApplyChangesError::OnDiskDeallocAccountError(
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
