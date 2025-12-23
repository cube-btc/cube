use crate::inscriptive::coin_manager::bodies::account_body::account_body::CMAccountBody;
use crate::inscriptive::coin_manager::bodies::contract_body::contract_body::CMContractBody;
use crate::inscriptive::coin_manager::bodies::contract_body::shadow_space::shadow_space::ShadowSpace;
use crate::inscriptive::coin_manager::delta::delta::CMDelta;
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
use serde_json::{Map, Value};
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
const ACCOUNT_BALANCE_SPECIAL_DB_KEY: [u8; 1] = [0x00; 1];

/// Special db key for the account shadow allocs sum value (0x01..).
const ACCOUNT_ALLOCS_SUM_SPECIAL_DB_KEY: [u8; 1] = [0x01; 1];

/// Special db key for the contract balance (0x00..).
const CONTRACT_BALANCE_SPECIAL_DB_KEY: [u8; 32] = [0x00; 32];

/// Special db key for the contract shadow allocs sum value (0x01..).
const CONTRACT_ALLOCS_SUM_SPECIAL_DB_KEY: [u8; 32] = [0x01; 32];

/// A database manager for handling account and contract balances & shadow space allocations.
pub struct CoinManager {
    // In-memory account & contract bodies.
    in_memory_accounts: HashMap<AccountKey, CMAccountBody>,
    in_memory_contracts: HashMap<ContractId, CMContractBody>,

    // On-disk accounts & contracts.
    on_disk_accounts: sled::Db,
    on_disk_contracts: sled::Db,

    // State differences to be applied.
    delta: CMDelta,

    // Backup of state differences in case of rollback.
    backup_of_delta: CMDelta,
}

/// Guarded 'CoinManager'.
#[allow(non_camel_case_types)]
pub type COIN_MANAGER = Arc<Mutex<CoinManager>>;

impl CoinManager {
    pub fn new(chain: Chain) -> Result<COIN_MANAGER, CMConstructionError> {
        // 1 Open the accounts db.
        let accounts_db_path = format!("storage/{}/coins/accounts", chain.to_string());
        let accounts_db = sled::open(accounts_db_path).map_err(|e| {
            CMConstructionError::AccountConstructionError(CMConstructionAccountError::DBOpenError(
                e,
            ))
        })?;

        // 2 Open the contracts db.
        let contracts_db_path = format!("storage/{}/coins/contracts", chain.to_string());
        let contracts_db = sled::open(contracts_db_path).map_err(|e| {
            CMConstructionError::ContractConstructionError(
                CMConstructionContractError::DBOpenError(e),
            )
        })?;

        // 3 Initialize the in-memory lists of account and contract bodies.
        let mut account_bodies = HashMap::<AccountKey, CMAccountBody>::new();
        let mut contract_bodies = HashMap::<ContractId, CMContractBody>::new();

        // 4 Collect account bodies from the account database.
        for tree_name in accounts_db.tree_names() {
            // 4.1 Deserialize account key bytes from tree name.
            let account_key: [u8; 32] = match tree_name.as_ref().try_into() {
                Ok(key) => key,
                Err(_) => {
                    // Tree name is probably '__sled__default'. Skip it.
                    continue;
                }
            };

            // 4.2 Open the tree.
            let tree = accounts_db.open_tree(tree_name).map_err(|e| {
                CMConstructionError::AccountConstructionError(
                    CMConstructionAccountError::TreeOpenError(account_key, e),
                )
            })?;

            // 4.3 Initialize the account balance and shadow allocs sum.
            let mut account_balance: u64 = 0;
            let mut account_global_shadow_allocs_sum: u128 = 0;

            // 4.4 Iterate over all items in the tree.
            for (index, item) in tree.iter().enumerate() {
                // 4.4.1 Get the key and value.
                let (key, value) = match item {
                    Ok((k, v)) => (k, v),
                    Err(e) => {
                        return Err(CMConstructionError::AccountConstructionError(
                            CMConstructionAccountError::TreeIterError(index, e),
                        ));
                    }
                };

                // 4.4.2 Deserialize the key byte.
                let tree_key_byte: [u8; 1] = key.as_ref().try_into().map_err(|_| {
                    CMConstructionError::AccountConstructionError(
                        CMConstructionAccountError::UnableToDeserializeKeyBytesFromTreeKey(
                            account_key,
                            index,
                            key.to_vec(),
                        ),
                    )
                })?;

                // 4.4.3 Match the tree key bytes.
                match tree_key_byte {
                    // 4.4.3.1 If the key is (0x00..), it is a special key that corresponds to the account balance value.
                    ACCOUNT_BALANCE_SPECIAL_DB_KEY => {
                        // 4.4.3.1.1 Deserialize the value bytes.
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

                        // 4.4.3.1.2 Update the account balance.
                        account_balance = account_balance_deserialized;
                    }
                    // 4.4.3.2 If the key is (0x01..), it is a special key that corresponds to the account shadow allocs sum value.
                    ACCOUNT_ALLOCS_SUM_SPECIAL_DB_KEY => {
                        // 4.4.3.2.1 Deserialize the value bytes.
                        let account_global_shadow_allocs_sum_deserialized: u128 =
                            u128::from_le_bytes(value.as_ref().try_into().map_err(|_| {
                                CMConstructionError::AccountConstructionError(CMConstructionAccountError::UnableToDeserializeAccountShadowAllocsSumFromTreeValue(
                                    account_key,
                                    index,
                                    tree_key_byte,
                                    value.to_vec(),
                                ))
                            })?);

                        // 4.4.3.2.2 Update the account global shadow allocs sum.
                        account_global_shadow_allocs_sum =
                            account_global_shadow_allocs_sum_deserialized;
                    }
                    _ => {
                        // 4.4.3.3 This key is a normal account key that corresponds to an account allocation.
                        return Err(CMConstructionError::AccountConstructionError(
                            CMConstructionAccountError::InvalidTreeKeyEncountered(
                                account_key,
                                tree_key_byte.to_vec(),
                            ),
                        ));
                    }
                }
            }

            // 4.5 Construct the account body.
            let account_body =
                CMAccountBody::new(account_balance, account_global_shadow_allocs_sum);

            // 4.6 Insert the account body into the account bodies list.
            account_bodies.insert(account_key, account_body);
        }

        // 5 Collect contract bodies from the contract database.
        for tree_name in contracts_db.tree_names() {
            // 5.1 Deserialize contract id bytes from tree name.
            let contract_id: [u8; 32] = match tree_name.as_ref().try_into() {
                Ok(key) => key,
                Err(_) => {
                    // Tree name is probably '__sled__default'. Skip it.
                    continue;
                }
            };

            // 5.2 Open the tree.
            let tree = contracts_db.open_tree(&tree_name).map_err(|e| {
                CMConstructionError::ContractConstructionError(
                    CMConstructionContractError::TreeOpenError(contract_id, e),
                )
            })?;

            // 5.3 Initialize the list of shadow space allocations.
            let mut allocs = HashMap::<AccountKey, SatiSatoshiAmount>::new();

            // 5.4 Initialize the allocs sum and contract balance.
            let mut allocs_sum: u64 = 0;
            let mut contract_balance: u64 = 0;

            // 5.5 Iterate over all items in the tree.
            for (index, item) in tree.iter().enumerate() {
                // 5.5.1 Get the key and value.
                let (key, value) = match item {
                    Ok((k, v)) => (k, v),
                    Err(e) => {
                        return Err(CMConstructionError::ContractConstructionError(
                            CMConstructionContractError::TreeIterError(contract_id, index, e),
                        ));
                    }
                };

                // 5.5.2 Deserialize the key bytes.
                let tree_key_bytes: [u8; 32] = key.as_ref().try_into().map_err(|_| {
                    CMConstructionError::ContractConstructionError(
                        CMConstructionContractError::UnableToDeserializeKeyBytesFromTreeKey(
                            contract_id,
                            index,
                            key.to_vec(),
                        ),
                    )
                })?;

                // 5.5.3 Match the tree key bytes.
                match tree_key_bytes {
                    // 5.5.3.1 If the key is (0x00..), it is a special key that corresponds to the contract balance value.
                    CONTRACT_BALANCE_SPECIAL_DB_KEY => {
                        // 5.5.3.1.1 Deserialize the value bytes.
                        let contract_balance_value_in_satoshis: u64 =
                                u64::from_le_bytes(value.as_ref().try_into().map_err(|_| {
                                    CMConstructionError::ContractConstructionError(CMConstructionContractError::UnableToDeserializeContractBalanceFromTreeValue(
                                        contract_id,
                                        index,
                                        tree_key_bytes,
                                        value.to_vec(),
                                    ))
                                })?);

                        // 5.5.3.1.2 Update the contract balance.
                        contract_balance = contract_balance_value_in_satoshis;
                    }
                    // 5.5.3.2 If the key is (0x01..), it is a special key that corresponds to the allocs sum value.
                    CONTRACT_ALLOCS_SUM_SPECIAL_DB_KEY => {
                        // 5.5.3.2.1 Deserialize the value bytes.
                        let allocs_sum_value_in_satoshis: u64 =
                                u64::from_le_bytes(value.as_ref().try_into().map_err(|_| {
                                    CMConstructionError::ContractConstructionError(CMConstructionContractError::UnableToDeserializeAllocsSumFromTreeValue(
                                        contract_id,
                                        index,
                                        tree_key_bytes,
                                        value.to_vec(),
                                    ))
                                })?);

                        // 5.5.3.2.2 Update the shadow space allocations sum.
                        allocs_sum = allocs_sum_value_in_satoshis;
                    }
                    _ => {
                        // 5.5.3.3 This key is an account key that corresponds to an allocation in the contract's shadow space.

                        // 5.5.3.3.1 Deserialize the allocation value in sati-satoshis.
                        let alloc_value_in_sati_satoshis: u128 =
                                u128::from_le_bytes(value.as_ref().try_into().map_err(|_| {
                                    CMConstructionError::ContractConstructionError(CMConstructionContractError::UnableToDeserializeAllocValueFromTreeValue(
                                        contract_id,
                                        index,
                                        tree_key_bytes,
                                        value.to_vec(),
                                    ))
                                })?);

                        // 5.5.3.3.2 Insert the allocation.
                        allocs.insert(tree_key_bytes, alloc_value_in_sati_satoshis);
                    }
                }
            }

            // 5.6 Check if the shadow space allocations sum exceeds the contract balance.
            if allocs_sum > contract_balance {
                return Err(CMConstructionError::ContractConstructionError(
                    CMConstructionContractError::AllocsSumExceedsTheContractBalance(
                        contract_id,
                        allocs_sum,
                        contract_balance,
                    ),
                ));
            }

            // 5.7 Construct the shadow space.
            let shadow_space = ShadowSpace::new(allocs_sum, allocs);

            // 5.8 Construct the contract body.
            let contract_body = CMContractBody::new(contract_balance, shadow_space);

            // 5.9 Insert the contract body into the contract bodies list.
            contract_bodies.insert(contract_id, contract_body);
        }

        // 6 Construct the coin holder.
        let coin_holder = CoinManager {
            in_memory_accounts: account_bodies,
            in_memory_contracts: contract_bodies,
            on_disk_accounts: accounts_db,
            on_disk_contracts: contracts_db,
            delta: CMDelta::fresh_new(),
            backup_of_delta: CMDelta::fresh_new(),
        };

        // 7 Guard the coin holder.
        let guarded_coin_holder = Arc::new(Mutex::new(coin_holder));

        // 8 Return the guarded coin holder.
        Ok(guarded_coin_holder)
    }

    /// Clones the deltas into the backup.   
    fn backup_delta(&mut self) {
        self.backup_of_delta = self.delta.clone();
    }

    /// Restores the deltas from the backup.
    fn restore_delta(&mut self) {
        self.delta = self.backup_of_delta.clone();
    }

    /// Returns the mutable ephemeral shadow space from delta.
    fn get_mut_ephemeral_contract_shadow_space(
        &mut self,
        contract_id: ContractId,
    ) -> Option<&mut ShadowSpace> {
        // 1 If the shadow space is not in the delta, create it.
        if !self.delta.updated_shadow_spaces.contains_key(&contract_id) {
            // 1.1 Get the contract body from the permanent in-memory states.
            let contract_body = self.in_memory_contracts.get(&contract_id)?;

            // 1.2 Get the shadow space.
            let shadow_space = contract_body.shadow_space.clone();

            // 1.3 Insert the shadow space into the delta.
            self.delta
                .updated_shadow_spaces
                .insert(contract_id, shadow_space);
        }

        // 2 Return the mutable ephemeral shadow space.
        self.delta.updated_shadow_spaces.get_mut(&contract_id)
    }

    /// Prepares 'CoinManager' prior to each execution.
    ///
    /// NOTE: Used by the Engine.
    pub fn pre_execution(&mut self) {
        // Backup the deltas.
        self.backup_delta();
    }

    /// Returns the account body for a given account key.
    pub fn get_account_body(&self, account_key: AccountKey) -> Option<CMAccountBody> {
        self.in_memory_accounts.get(&account_key).cloned()
    }

    /// Returns the contract body for a given contract ID.
    pub fn get_contract_body(&self, contract_id: ContractId) -> Option<CMContractBody> {
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
        // 1 Try to get from the delta first.
        if let Some(value) = self.delta.updated_account_balances.get(&account_key) {
            return Some(value.clone());
        }

        // 2 And then try to get from the permanent in-memory states.
        self.in_memory_accounts
            .get(&account_key)
            .map(|account_body| account_body.balance)
    }

    /// Returns a contract's balance in satoshis.
    pub fn get_contract_balance(&self, contract_id: ContractId) -> Option<u64> {
        // 1 Try to get from the delta first.
        if let Some(value) = self.delta.updated_contract_balances.get(&contract_id) {
            return Some(value.clone());
        }

        // 2 And then try to get from the permanent in-memory states.
        self.in_memory_contracts
            .get(&contract_id)
            .map(|contract_body| contract_body.balance)
    }

    /// Returns the base sum of a given account's shadow allocation values across all contracts in sati-satoshis.
    /// This does NOT account for deferred proportional changes (shadow_up_all/down_all).
    fn get_account_global_shadow_allocs_sum_in_sati_satoshis_base(
        &self,
        account_key: AccountKey,
    ) -> Option<u128> {
        // 1 Try to get from the delta first.
        if let Some(value) = self
            .delta
            .updated_global_shadow_allocs_sums
            .get(&account_key)
        {
            return Some(value.clone());
        }

        // 2 And then try to get from the permanent in-memory states.
        self.in_memory_accounts
            .get(&account_key)
            .map(|account_body| account_body.global_shadow_allocs_sum)
    }

    /// Returns the sum of a given account's shadow allocation values across all contracts in satoshis.
    /// This does NOT account for deferred proportional changes (shadow_up_all/down_all).
    pub fn get_account_global_shadow_allocs_sum_in_satoshis_base(
        &self,
        account_key: AccountKey,
    ) -> Option<u64> {
        // 1 Get the sati-satoshi value.
        let sati_satoshi_value =
            self.get_account_global_shadow_allocs_sum_in_sati_satoshis_base(account_key)?;

        // 2 Convert to satoshi value.
        let satoshi_value = sati_satoshi_value / ONE_SATOSHI_IN_SATI_SATOSHIS;

        // 3 Return the result.
        Some(satoshi_value as u64)
    }

    /// Returns the sum of a given account's shadow allocation values across all contracts in sati-satoshis.
    /// This accounts for deferred proportional changes (shadow_up_all/down_all) in shadow spaces.
    ///
    /// NOTE: Used for tests only.
    pub fn get_account_global_shadow_allocs_sum_in_sati_satoshis(
        &self,
        account_key: AccountKey,
    ) -> Option<u128> {
        // 1 Get the base global shadow allocs sum (without deferred changes).
        let base_global_shadow_allocs_sum =
            self.get_account_global_shadow_allocs_sum_in_sati_satoshis_base(account_key)?;

        // 2 Calculate the sum of deferred proportional changes from all shadow spaces.
        let mut deferred_changes_sum: i128 = 0;

        for (_contract_id, shadow_space) in self.delta.updated_shadow_spaces.iter() {
            // 2.1 Check if there's a deferred proportional change in this shadow space.
            let deferred_change_in_satoshis = shadow_space.shadow_up_all_down_alls;
            if deferred_change_in_satoshis == 0 {
                continue;
            }

            // 2.2 Check if this account has an allocation in this shadow space.
            let base_alloc_value_in_sati_satoshis = match shadow_space.allocs.get(&account_key) {
                Some(value) => *value,
                None => continue, // Account doesn't have an allocation in this contract, skip.
            };

            // 2.3 Calculate the base allocs_sum (before deferred changes).
            let current_allocs_sum_in_satoshis = shadow_space.allocs_sum;
            let base_allocs_sum_in_satoshis =
                (current_allocs_sum_in_satoshis as i64 - deferred_change_in_satoshis) as u64;

            // 2.4 Check if base allocs sum is zero (cannot compute proportions).
            if base_allocs_sum_in_satoshis == 0 {
                continue;
            }

            // 2.5 Convert values to sati-satoshis for calculation.
            let base_allocs_sum_in_sati_satoshis =
                (base_allocs_sum_in_satoshis as u128) * ONE_SATOSHI_IN_SATI_SATOSHIS;
            let deferred_change_in_sati_satoshis =
                (deferred_change_in_satoshis.abs() as u128) * ONE_SATOSHI_IN_SATI_SATOSHIS;

            // 2.6 Calculate the proportional change for this account in this contract.
            let individual_change_in_sati_satoshis = if deferred_change_in_satoshis > 0 {
                // Up_all: proportional increase
                (base_alloc_value_in_sati_satoshis * deferred_change_in_sati_satoshis)
                    / base_allocs_sum_in_sati_satoshis
            } else {
                // Down_all: proportional decrease
                let individual_down = (base_alloc_value_in_sati_satoshis
                    * deferred_change_in_sati_satoshis)
                    / base_allocs_sum_in_sati_satoshis;
                // Ensure we don't go below zero.
                individual_down.min(base_alloc_value_in_sati_satoshis)
            };

            // 2.7 Add the change to the sum (positive for up_all, negative for down_all).
            if individual_change_in_sati_satoshis > 0 {
                if deferred_change_in_satoshis > 0 {
                    deferred_changes_sum += individual_change_in_sati_satoshis as i128;
                } else {
                    deferred_changes_sum -= individual_change_in_sati_satoshis as i128;
                }
            }
        }

        // 3 Apply the deferred changes sum to the base value.
        if deferred_changes_sum == 0 {
            return Some(base_global_shadow_allocs_sum);
        }

        let effective_global_shadow_allocs_sum = if deferred_changes_sum > 0 {
            base_global_shadow_allocs_sum
                .checked_add(deferred_changes_sum as u128)
                .expect(
                    "Account global shadow allocs sum overflow on deferred proportional changes",
                )
        } else {
            base_global_shadow_allocs_sum
                .checked_sub((-deferred_changes_sum) as u128)
                .expect(
                    "Account global shadow allocs sum underflow on deferred proportional changes",
                )
        };

        Some(effective_global_shadow_allocs_sum)
    }

    /// Returns the sum of a given account's shadow allocation values across all contracts in satoshis.
    ///
    /// NOTE: Used for tests only.
    pub fn get_account_global_shadow_allocs_sum_in_satoshis(
        &self,
        account_key: AccountKey,
    ) -> Option<u64> {
        // 1 Get the sati-satoshi value.
        let sati_satoshi_value =
            self.get_account_global_shadow_allocs_sum_in_sati_satoshis(account_key)?;

        // 2 Convert to satoshi value.
        let satoshi_value = sati_satoshi_value / ONE_SATOSHI_IN_SATI_SATOSHIS;

        // 3 Return the result.
        Some(satoshi_value as u64)
    }

    /// Returns the sum of all shadow allocation values of a given contract's shadow space in satoshis.
    pub fn get_contract_shadow_allocs_sum_in_satoshis(&self, contract_id: [u8; 32]) -> Option<u64> {
        // 1 Try to read from the delta first.
        if let Some(allocs_sum) = self.delta.updated_shadow_spaces.get(&contract_id) {
            return Some(allocs_sum.allocs_sum);
        }

        // 2 And then try to get from the in-memory states.
        self.in_memory_contracts
            .get(&contract_id)
            .map(|body| body.shadow_space.allocs_sum)
    }

    /// Returns the number of total shadow allocations of a given contract's shadow space.
    pub fn get_contract_num_shadow_allocs(&self, contract_id: [u8; 32]) -> Option<u64> {
        // 1 Try to get from the delta first.
        if let Some(shadow_space) = self.delta.updated_shadow_spaces.get(&contract_id) {
            return Some(shadow_space.allocs.len() as u64);
        }

        // 2 And then try to get from the in-memory states.
        self.in_memory_contracts
            .get(&contract_id)
            .map(|body| body.shadow_space.allocs.len() as u64)
    }

    /// Returns the base shadow allocation value (without deferred proportional changes) of a given account for a given contract in sati-satoshis.
    ///
    /// NOTE: This is the internal version used by shadow_up/shadow_down operations that need to work with base values.
    fn get_shadow_alloc_value_in_sati_satoshis_base(
        &self,
        contract_id: [u8; 32],
        account_key: AccountKey,
    ) -> Option<u128> {
        // 1 Check if the account is epheremally deallocated in the delta.
        if let Some(dealloc_list) = self.delta.deallocs_list.get(&contract_id) {
            if dealloc_list.contains(&account_key) {
                // 1.1 The account is epheremally deallocated in the same execution.
                // 1.2 Therefore, there is no allocation value anymore to return.
                return None;
            }
        }

        // 2 Try to read from the delta first (base value only, without deferred proportional changes).
        if let Some(shadow_space) = self.delta.updated_shadow_spaces.get(&contract_id) {
            return shadow_space.allocs.get(&account_key).cloned();
        }

        // 3 And then try to read from the permanent states.
        self.in_memory_contracts
            .get(&contract_id)
            .and_then(|body| body.shadow_space.allocs.get(&account_key).cloned())
    }

    /// Returns the shadow allocation value of a given account for a given contract in sati-satoshis.
    ///
    /// NOTE: This version accounts for deferred proportional changes (shadow_up_all/down_all).
    pub fn get_shadow_alloc_value_in_sati_satoshis(
        &self,
        contract_id: [u8; 32],
        account_key: AccountKey,
    ) -> Option<u128> {
        // 1 Check if the account is epheremally deallocated in the delta.
        if let Some(dealloc_list) = self.delta.deallocs_list.get(&contract_id) {
            if dealloc_list.contains(&account_key) {
                // 1.1 The account is epheremally deallocated in the same execution.
                // 1.2 Therefore, there is no allocation value anymore to return.
                return None;
            }
        }

        // 2 Try to read from the delta first.
        if let Some(shadow_space) = self.delta.updated_shadow_spaces.get(&contract_id) {
            // 2.1 Get the deferred change in satoshis.
            let deferred_change_in_satoshis = shadow_space.shadow_up_all_down_alls;

            // 2.2 Get the base allocation value.
            let base_alloc_value_in_sati_satoshis =
                shadow_space.allocs.get(&account_key).cloned()?;

            // 2.3 Check if there's a deferred proportional change to apply.
            if deferred_change_in_satoshis == 0 {
                // 2.3.1 No deferred change, return the base value directly.
                return Some(base_alloc_value_in_sati_satoshis);
            }

            // 2.4 Calculate the base allocs_sum (before deferred changes).
            let current_allocs_sum_in_satoshis = shadow_space.allocs_sum;

            // 2.5 Calculate the base allocs_sum (before deferred changes).
            let base_allocs_sum_in_satoshis =
                (current_allocs_sum_in_satoshis as i64 - deferred_change_in_satoshis) as u64;

            // 2.6 Check if the base allocs sum is zero (cannot compute proportions).
            if base_allocs_sum_in_satoshis == 0 {
                // 2.6.1 If base is zero, return the base value (no proportional change can be applied).
                return Some(base_alloc_value_in_sati_satoshis);
            }

            // 2.7 Convert values to sati-satoshis for calculation (matching apply_changes logic).
            let base_allocs_sum_in_sati_satoshis =
                (base_allocs_sum_in_satoshis as u128) * ONE_SATOSHI_IN_SATI_SATOSHIS;

            // 2.8 Convert the deferred change in satoshis to sati-satoshis.
            let deferred_change_in_sati_satoshis =
                (deferred_change_in_satoshis.abs() as u128) * ONE_SATOSHI_IN_SATI_SATOSHIS;

            // 2.9 Calculate the proportional change for this account (matching apply_changes logic).
            let individual_change_in_sati_satoshis = if deferred_change_in_satoshis > 0 {
                // Up_all: proportional increase
                (base_alloc_value_in_sati_satoshis * deferred_change_in_sati_satoshis)
                    / base_allocs_sum_in_sati_satoshis
            } else {
                // Down_all: proportional decrease
                let individual_down = (base_alloc_value_in_sati_satoshis
                    * deferred_change_in_sati_satoshis)
                    / base_allocs_sum_in_sati_satoshis;
                // Ensure we don't go below zero (matching apply_changes clamping).
                individual_down.min(base_alloc_value_in_sati_satoshis)
            };

            // 2.10 Calculate the new alloc value (matching apply_changes logic).
            let effective_alloc_value_in_sati_satoshis = if deferred_change_in_satoshis > 0 {
                base_alloc_value_in_sati_satoshis + individual_change_in_sati_satoshis
            } else {
                base_alloc_value_in_sati_satoshis.saturating_sub(individual_change_in_sati_satoshis)
            };

            // 2.11 Return the effective value.
            return Some(effective_alloc_value_in_sati_satoshis);
        }

        // 3 And then try to read from the permanent states (no deferred changes in permanent state).
        self.in_memory_contracts
            .get(&contract_id)
            .and_then(|body| body.shadow_space.allocs.get(&account_key).cloned())
    }

    /// Returns the shadow allocation value of a given account for a given contract in satoshis.
    pub fn get_shadow_alloc_value_in_satoshis(
        &self,
        contract_id: [u8; 32],
        account_key: AccountKey,
    ) -> Option<u64> {
        // 1 Get the sati-satoshi value.
        let sati_satoshi_value =
            self.get_shadow_alloc_value_in_sati_satoshis(contract_id, account_key)?;

        // 2 Convert to satoshi value.
        let satoshi_value = sati_satoshi_value / ONE_SATOSHI_IN_SATI_SATOSHIS;

        // 3 Return the result.
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
        // 1 Check if the account key collides with reserved database keys.
        if account_key == CONTRACT_BALANCE_SPECIAL_DB_KEY
            || account_key == CONTRACT_ALLOCS_SUM_SPECIAL_DB_KEY
        {
            return Err(CMRegisterAccountError::AccountKeyCannotBeTheSpecialDbKeys(
                account_key,
            ));
        }

        // 2 Check if the account has just been epheremally registered in the delta.
        if self
            .delta
            .new_accounts_to_register
            .contains_key(&account_key)
        {
            return Err(
                CMRegisterAccountError::AccountHasJustBeenEphemerallyRegistered(account_key),
            );
        }

        // 3 Check if the account is already permanently registered.
        if self.is_account_registered(account_key) {
            return Err(CMRegisterAccountError::AccountIsAlreadyPermanentlyRegistered(account_key));
        }

        // 4 Insert into the new accounts to register list in the delta.
        self.delta
            .new_accounts_to_register
            .insert(account_key, initial_account_balance);

        // 5 Return the result.
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
        // 1 Check if the contract has just been epheremally registered in the delta.
        if self
            .delta
            .new_contracts_to_register
            .contains_key(&contract_id)
        {
            return Err(
                CMRegisterContractError::ContractHasJustBeenEphemerallyRegistered(contract_id),
            );
        }

        // 2 Check if the contract is already permanently registered.
        if self.is_contract_registered(contract_id) {
            return Err(
                CMRegisterContractError::ContractIsAlreadyPermanentlyRegistered(contract_id),
            );
        }

        // 3 Insert into the new contracts to register list in the delta.
        self.delta
            .new_contracts_to_register
            .insert(contract_id, initial_contract_balance);

        // 4 Return the result.
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
        // 1 Get the account's existing balance.
        let account_balance_in_satoshis: u64 = self.get_account_balance(account_key).ok_or(
            CMAccountBalanceUpError::UnableToGetAccountBalance(account_key),
        )?;

        // 2 Calculate the new account balance.
        let new_account_balance_in_satoshis: u64 =
            account_balance_in_satoshis + up_value_in_satoshis;

        // 3 Epheremally update the account's balance.
        self.delta
            .epheremally_update_account_balance(account_key, new_account_balance_in_satoshis);

        // 4 Return the result.
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
        // 1 Get the account's existing balance.
        let account_balance_in_satoshis: u64 = self.get_account_balance(account_key).ok_or(
            CMAccountBalanceDownError::UnableToGetAccountBalance(account_key),
        )?;

        // 2 Check if the decrease would make the account balance go below zero.
        if down_value_in_satoshis > account_balance_in_satoshis {
            return Err(CMAccountBalanceDownError::AccountBalanceWouldGoBelowZero(
                account_key,
                account_balance_in_satoshis,
                down_value_in_satoshis,
            ));
        }

        // 3 Calculate the new account balance.
        let new_account_balance_in_satoshis: u64 =
            account_balance_in_satoshis - down_value_in_satoshis;

        // 4 Epheremally update the account's balance.
        self.delta
            .epheremally_update_account_balance(account_key, new_account_balance_in_satoshis);

        // 5 Return the result.
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
        // 1 Get the contract's existing balance.
        let existing_contract_balance_in_satoshis: u64 =
            self.get_contract_balance(contract_id).ok_or(
                CMContractBalanceUpError::UnableToGetContractBalance(contract_id),
            )?;

        // 2 Calculate the contract's new balance.
        let new_contract_balance_in_satoshis: u64 =
            existing_contract_balance_in_satoshis + up_value_in_satoshis;

        // 3 Epheremally update the contract's balance.
        self.delta
            .epheremally_update_contract_balance(contract_id, new_contract_balance_in_satoshis);

        // 4 Return the result.
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
        // 1 Get the contract's existing balance.
        let existing_contract_balance_in_satoshis: u64 =
            self.get_contract_balance(contract_id).ok_or(
                CMContractBalanceDownError::UnableToGetContractBalance(contract_id),
            )?;

        // 2 Check if the decrease would make the contract balance go below zero.
        if down_value_in_satoshis > existing_contract_balance_in_satoshis {
            return Err(CMContractBalanceDownError::ContractBalanceWouldGoBelowZero(
                contract_id,
                existing_contract_balance_in_satoshis,
                down_value_in_satoshis,
            ));
        }

        // 3 Calculate the contract's new balance.
        let new_contract_balance_in_satoshis: u64 =
            existing_contract_balance_in_satoshis - down_value_in_satoshis;

        // 4 Get the contract's existing shadow allocs sum.
        let existing_contract_shadow_allocs_sum_in_satoshis: u64 = self
            .get_contract_shadow_allocs_sum_in_satoshis(contract_id)
            .ok_or(CMContractBalanceDownError::UnableToGetContractAllocsSum(
                contract_id,
            ))?;

        // 5 Check if the contract balance would go below the allocs sum.
        // 5.1 Shadow allocs sum is bound by the contract's balance.
        if new_contract_balance_in_satoshis < existing_contract_shadow_allocs_sum_in_satoshis {
            return Err(
                CMContractBalanceDownError::ContractBalanceWouldGoBelowAllocsSum(
                    contract_id,
                    new_contract_balance_in_satoshis,
                    existing_contract_shadow_allocs_sum_in_satoshis,
                ),
            );
        }

        // 6 Epheremally update the contract's balance.
        self.delta
            .epheremally_update_contract_balance(contract_id, new_contract_balance_in_satoshis);

        // 7 Return the result.
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
        // 1 Check if the account has just been epheremally allocated in the delta.
        // 1.1 We do not allow it to be allocated again in the same execution.
        if let Some(allocs_list) = self.delta.allocs_list.get(&contract_id) {
            if allocs_list.contains(&account_key) {
                return Err(
                    CMContractShadowAllocAccountError::AccountHasJustBeenEphemerallyAllocated(
                        contract_id,
                        account_key,
                    ),
                );
            }
        }

        // 2 Check if the account has just been epheremally deallocated in the delta.
        // 2.1 We do not allow it to be allocated after being deallocated in the same execution.
        if let Some(deallocs_list) = self.delta.deallocs_list.get(&contract_id) {
            if deallocs_list.contains(&account_key) {
                return Err(
                    CMContractShadowAllocAccountError::AccountHasJustBeenEphemerallyDeallocated(
                        contract_id,
                        account_key,
                    ),
                );
            }
        }

        // 3 Check if the account key is already permanently allocated by reading its allocation value.
        // 3.1 We do not allow it to be allocated again if already permanently allocated.
        // 3.2 Use base version to check the actual stored value (without deferred proportional changes).
        if self
            .get_shadow_alloc_value_in_sati_satoshis_base(contract_id, account_key)
            .is_some()
        {
            return Err(
                CMContractShadowAllocAccountError::AccountIsAlreadyPermanentlyAllocated(
                    contract_id,
                    account_key,
                ),
            );
        }

        // 4 Epheremally insert the new allocation to the shadow space.
        {
            // 4.1 Get mutable ephemeral shadow space from the delta.
            let mut_epheremal_shadow_space = self
                .get_mut_ephemeral_contract_shadow_space(contract_id)
                .ok_or(
                    CMContractShadowAllocAccountError::UnableToGetMutEphemeralShadowSpace(
                        contract_id,
                    ),
                )?;

            // 4.2 Epheremally insert the new allocation with value initially set to zero.
            mut_epheremal_shadow_space.insert_update_alloc(account_key, 0);
        }

        // 5 Epheremally insert the allocation record to the allocs list.
        self.delta
            .epheremally_insert_alloc(contract_id, account_key);

        // 6 Return the result.
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
        // 1 Check if the account has just been epheremally allocated in the delta.
        // 1.1 We do not allow it to be deallocated if it is just allocated in the same execution.
        if let Some(allocs_list) = self.delta.allocs_list.get(&contract_id) {
            if allocs_list.contains(&account_key) {
                return Err(
                    CMContractShadowDeallocAccountError::AccountHasJustBeenEphemerallyAllocated(
                        contract_id,
                        account_key,
                    ),
                );
            }
        }

        // 2 Check if the account has just been epheremally deallocated in the delta.
        if let Some(deallocs_list) = self.delta.deallocs_list.get(&contract_id) {
            if deallocs_list.contains(&account_key) {
                return Err(
                    CMContractShadowDeallocAccountError::AccountHasJustBeenEphemerallyDeallocated(
                        contract_id,
                        account_key,
                    ),
                );
            }
        }

        // 3 Get the account's allocation value in sati-satoshis.
        // 3.1 This also checks if the account is acutally permanently allocated.
        // 3.2 Use base version to get the actual stored value (without deferred proportional changes).
        let allocation_value_in_sati_satoshis = self
            .get_shadow_alloc_value_in_sati_satoshis_base(contract_id, account_key)
            .ok_or(
                CMContractShadowDeallocAccountError::UnableToGetAccountAllocValue(
                    contract_id,
                    account_key,
                ),
            )?;

        // 4 Check if the account allocation value is non-zero.
        // 4.1 Deallocation is allowed only if the allocation value is zero.
        if allocation_value_in_sati_satoshis != 0 {
            return Err(CMContractShadowDeallocAccountError::AllocValueIsNonZero(
                contract_id,
                account_key,
            ));
        }

        // 5 Epheremally remove the account from the shadow space.
        {
            // 5.1 Get mutable ephemeral shadow space from the delta.
            let mut_epheremal_shadow_space = self
                .get_mut_ephemeral_contract_shadow_space(contract_id)
                .ok_or(
                    CMContractShadowDeallocAccountError::UnableToGetMutEphemeralShadowSpace(
                        contract_id,
                    ),
                )?;

            // 5.2 Epheremally remove the account key from the shadow space.
            mut_epheremal_shadow_space.remove_alloc(account_key);
        }

        // 6 Epheremally insert the deallocation record to the deallocs list.
        self.delta
            .epheremally_insert_dealloc(contract_id, account_key);

        // 7 Return the result.
        Ok(())
    }

    /// Increases an account's global shadow allocs sum value.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    fn account_global_shadow_allocs_sum_up(
        &mut self,
        account_key: AccountKey,
        up_value_in_sati_satoshis: u128,
    ) -> Result<(), CMAccountShadowAllocsSumUpError> {
        // 1 Get the existing account global shadow allocs sum in sati-satoshis (base value, without deferred changes).
        let account_global_shadow_allocs_sum_in_sati_satoshis: u128 = self
            .get_account_global_shadow_allocs_sum_in_sati_satoshis_base(account_key)
            .ok_or(
                CMAccountShadowAllocsSumUpError::UnableToGetAccountShadowAllocsSum(account_key),
            )?;

        // 2 Calculate the new value.
        let new_account_global_shadow_allocs_sum_in_sati_satoshis: u128 =
            account_global_shadow_allocs_sum_in_sati_satoshis + up_value_in_sati_satoshis;

        // 3 Epheremally update the account's global shadow allocs sum.
        self.delta
            .epheremally_update_account_global_shadow_allocs_sum(
                account_key,
                new_account_global_shadow_allocs_sum_in_sati_satoshis,
            );

        // 4 Return the result.
        Ok(())
    }

    /// Decreases an account's global shadow allocs sum value.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    fn account_global_shadow_allocs_sum_down(
        &mut self,
        account_key: AccountKey,
        down_value_in_sati_satoshis: u128,
    ) -> Result<(), CMAccountShadowAllocsSumDownError> {
        // 1 Get the old ephemeral account global shadow allocs sum before any mutable borrows (base value, without deferred changes).
        let account_global_shadow_allocs_sum_in_sati_satoshis: u128 = self
            .get_account_global_shadow_allocs_sum_in_sati_satoshis_base(account_key)
            .ok_or(
                CMAccountShadowAllocsSumDownError::UnableToGetAccountShadowAllocsSum(account_key),
            )?;

        // 2 Check if the decrease would make the account global shadow allocs sum go below zero.
        if down_value_in_sati_satoshis > account_global_shadow_allocs_sum_in_sati_satoshis {
            return Err(
                CMAccountShadowAllocsSumDownError::AccountShadowAllocsSumWouldGoBelowZero(
                    account_key,
                    account_global_shadow_allocs_sum_in_sati_satoshis,
                    down_value_in_sati_satoshis,
                ),
            );
        }

        // 3 Calculate the new ephemeral account global shadow allocs sum.
        let new_account_global_shadow_allocs_sum_in_sati_satoshis: u128 =
            account_global_shadow_allocs_sum_in_sati_satoshis - down_value_in_sati_satoshis;

        // 4 Epheremally update the account's global shadow allocs sum.
        self.delta
            .epheremally_update_account_global_shadow_allocs_sum(
                account_key,
                new_account_global_shadow_allocs_sum_in_sati_satoshis,
            );

        // 5 Return the result.
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
        // 1 Convert the increase value to sati-satoshi value.
        let up_value_in_sati_satoshis: u128 =
            (up_value_in_satoshis as u128) * ONE_SATOSHI_IN_SATI_SATOSHIS;

        // 2 Get the account's existing shadow allocation value for this contract.
        // 2.1 Use base version to get the actual stored value (without deferred proportional changes),
        //     since we will modify it directly.
        let account_shadow_alloc_value_in_sati_satoshis: u128 = self
            .get_shadow_alloc_value_in_sati_satoshis_base(contract_id, account_key)
            .ok_or(CMShadowUpError::UnableToGetAccountShadowAllocValue(
                contract_id,
                account_key,
            ))?;

        // 3 Calculate the account's new shadow allocation value.
        let new_account_shadow_alloc_value_in_sati_satoshis: u128 =
            account_shadow_alloc_value_in_sati_satoshis + up_value_in_sati_satoshis;

        // 4 Get the contract's existing balance.
        let contract_balance_in_satoshis: u64 = self
            .get_contract_balance(contract_id)
            .ok_or(CMShadowUpError::UnableToGetContractBalance(contract_id))?;

        // 5 Get mutable ephemeral shadow space from the delta.
        let mut_epheremal_shadow_space = self
            .get_mut_ephemeral_contract_shadow_space(contract_id)
            .ok_or(CMShadowUpError::UnableToGetMutEphemeralShadowSpace(
                contract_id,
            ))?;

        // 6 Calculate the contract's new shadow allocs sum value.
        let new_contract_allocs_sum_value_in_satoshis: u64 =
            mut_epheremal_shadow_space.allocs_sum + up_value_in_satoshis;

        // 7 Check if the contract's new shadow allocs sum value exceeds the contract balance.
        if new_contract_allocs_sum_value_in_satoshis > contract_balance_in_satoshis {
            return Err(CMShadowUpError::AllocsSumExceedsTheContractBalance(
                contract_id,
                new_contract_allocs_sum_value_in_satoshis,
                contract_balance_in_satoshis,
            ));
        }

        // 8 Epheremally update the account's shadow alloc value.
        mut_epheremal_shadow_space
            .insert_update_alloc(account_key, new_account_shadow_alloc_value_in_sati_satoshis);

        // 9 Epheremally update the contract's shadow allocs sum value.
        mut_epheremal_shadow_space.update_allocs_sum(new_contract_allocs_sum_value_in_satoshis);

        // 10 Update the account global shadow allocs sum value.
        {
            self.account_global_shadow_allocs_sum_up(account_key, up_value_in_sati_satoshis)
                .map_err(|error| {
                    CMShadowUpError::AccountShadowAllocsSumUpError(contract_id, account_key, error)
                })?;
        }

        // 11 Return the result.
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
        // 1 Convert the decrease value to sati-satoshi value.
        let down_value_in_sati_satoshis: u128 =
            (down_value_in_satoshis as u128) * ONE_SATOSHI_IN_SATI_SATOSHIS;

        // 2 Get the account's existing shadow alloc value for this contract.
        // 2.1 Use base version to get the actual stored value (without deferred proportional changes),
        //     since we will modify it directly.
        let account_shadow_alloc_value_in_sati_satoshis: u128 = self
            .get_shadow_alloc_value_in_sati_satoshis_base(contract_id, account_key)
            .ok_or(CMShadowDownError::UnableToGetAccountShadowAllocValue(
                contract_id,
                account_key,
            ))?;

        // 3 Check if the decrease would make the account's alloc value to go below zero.
        if down_value_in_sati_satoshis > account_shadow_alloc_value_in_sati_satoshis {
            return Err(CMShadowDownError::AccountShadowAllocValueWouldGoBelowZero(
                contract_id,
                account_key,
                account_shadow_alloc_value_in_sati_satoshis,
                down_value_in_sati_satoshis,
            ));
        }

        // 4 Calculate the account's new shadow alloc value.
        let new_account_shadow_alloc_value_in_sati_satoshis: u128 =
            account_shadow_alloc_value_in_sati_satoshis - down_value_in_sati_satoshis;

        // 5 Get mutable ephemeral shadow space from the delta.
        let mut_epheremal_shadow_space = self
            .get_mut_ephemeral_contract_shadow_space(contract_id)
            .ok_or(CMShadowDownError::UnableToGetMutEphemeralShadowSpace(
                contract_id,
            ))?;

        // 5 Get the contract's existing shadow allocs sum value.
        let contract_shadow_allocs_sum_in_satoshis: u64 = mut_epheremal_shadow_space.allocs_sum;

        // 6 Check if the decrease would make the contract's shadow allocs sum to go below zero.
        // NOTE: This is unlikely to happen, but we are checking for it just in case.
        if down_value_in_satoshis > contract_shadow_allocs_sum_in_satoshis {
            return Err(CMShadowDownError::ContractShadowAllocsSumWouldGoBelowZero(
                contract_id,
                contract_shadow_allocs_sum_in_satoshis,
                down_value_in_satoshis,
            ));
        }

        // 7 Calculate the contract's new shadow allocs sum value.
        let new_contract_allocs_sum_value_in_satoshis: u64 =
            contract_shadow_allocs_sum_in_satoshis - down_value_in_satoshis;

        // 8 Epheremally update the account's shadow alloc value.
        mut_epheremal_shadow_space
            .insert_update_alloc(account_key, new_account_shadow_alloc_value_in_sati_satoshis);

        // 9 Epheremally update the contract's shadow allocs sum value.
        mut_epheremal_shadow_space.update_allocs_sum(new_contract_allocs_sum_value_in_satoshis);

        // 10 Epheremally update the account global shadow allocs sum value.
        {
            self.account_global_shadow_allocs_sum_down(account_key, down_value_in_sati_satoshis)
                .map_err(|error| {
                    CMShadowDownError::AccountShadowAllocsSumDownError(
                        contract_id,
                        account_key,
                        error,
                    )
                })?;
        }

        // 11 Return the result.
        Ok(())
    }

    /// Proportionaly increases the shadow allocation value of all accounts in a contract shadow space by a given value.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    /// NOTE: The proportional calculation is deferred until `apply_changes` is called for efficiency.
    pub fn shadow_up_all(
        &mut self,
        contract_id: [u8; 32],
        up_value_in_satoshis: u64,
    ) -> Result<u64, CMShadowUpAllError> {
        // 1 Get the contract's existing balance.
        let contract_balance_in_satoshis: u64 = self
            .get_contract_balance(contract_id)
            .ok_or(CMShadowUpAllError::UnableToGetContractBalance(contract_id))?;

        // 2 Get the contract's existing shadow allocs sum value.
        let contract_shadow_allocs_sum_value_in_satoshis: u64 = self
            .get_contract_shadow_allocs_sum_in_satoshis(contract_id)
            .ok_or(CMShadowUpAllError::UnableToGetContractAllocsSum(
                contract_id,
            ))?;

        // 3 Check if the contract allocs sum is zero.
        // 3.1 This operation is not possible with zero allocs sum.
        if contract_shadow_allocs_sum_value_in_satoshis == 0 {
            return Err(CMShadowUpAllError::OperationNotPossibleWithZeroAllocsSum(
                contract_id,
            ));
        }

        // 4 Calculate the new contract allocs sum value (after applying this change).
        let new_contract_allocs_sum_value_in_satoshis: u64 =
            contract_shadow_allocs_sum_value_in_satoshis + up_value_in_satoshis;

        // 5 Check if the new contract alloc sum value exceeds the contract balance.
        if new_contract_allocs_sum_value_in_satoshis > contract_balance_in_satoshis {
            return Err(CMShadowUpAllError::AllocsSumExceedsTheContractBalance(
                contract_id,
                new_contract_allocs_sum_value_in_satoshis,
                contract_balance_in_satoshis,
            ));
        }

        // 6 Get the mutable ephemeral shadow space from the delta.
        let mut_epheremal_shadow_space = self
            .get_mut_ephemeral_contract_shadow_space(contract_id)
            .ok_or(CMShadowUpAllError::UnableToGetMutEphemeralShadowSpace(
                contract_id,
            ))?;

        // 7 Update the allocs_sum immediately (for validation in subsequent operations).
        mut_epheremal_shadow_space.update_allocs_sum(new_contract_allocs_sum_value_in_satoshis);

        // 8 Accumulate the deferred proportional change (positive value for up_all).
        mut_epheremal_shadow_space.add_deferred_proportional_change(up_value_in_satoshis as i64);

        // 9 Get the number of affected accounts (for return value).
        // 9.1 Count accounts that are not ephemerally deallocated.
        let num_affected_accounts = mut_epheremal_shadow_space.allocs.len() as u64;

        // 10 Return the number of affected accounts.
        Ok(num_affected_accounts)
    }

    /// Proportionaly decreases the shadow allocation value of all accounts in a contract shadow space by a given value.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    /// NOTE: The proportional calculation is deferred until `apply_changes` is called for efficiency.
    pub fn shadow_down_all(
        &mut self,
        contract_id: [u8; 32],
        down_value_in_satoshis: u64,
    ) -> Result<u64, CMShadowDownAllError> {
        // 1 Get the contract's existing balance.
        let contract_balance_in_satoshis: u64 = self.get_contract_balance(contract_id).ok_or(
            CMShadowDownAllError::UnableToGetContractBalance(contract_id),
        )?;

        // 2 Get the contract's existing shadow allocs sum value.
        let existing_contract_allocs_sum_in_satoshis: u64 = self
            .get_contract_shadow_allocs_sum_in_satoshis(contract_id)
            .ok_or(CMShadowDownAllError::UnableToGetContractAllocsSum(
                contract_id,
            ))?;

        // 3 Check if the contract allocs sum is zero.
        // 3.1 This operation is not possible with zero allocs sum.
        if existing_contract_allocs_sum_in_satoshis == 0 {
            return Err(CMShadowDownAllError::OperationNotPossibleWithZeroAllocsSum(
                contract_id,
            ));
        }

        // 4 Check if the decrease would make the allocs sum go below zero.
        if down_value_in_satoshis > existing_contract_allocs_sum_in_satoshis {
            return Err(CMShadowDownAllError::AllocsSumWouldGoBelowZero(
                contract_id,
                existing_contract_allocs_sum_in_satoshis,
                down_value_in_satoshis,
            ));
        }

        // 5 Calculate the new contract allocs sum value (after applying this change).
        let new_contract_allocs_sum_value_in_satoshis: u64 =
            existing_contract_allocs_sum_in_satoshis - down_value_in_satoshis;

        // 6 Check if the new contract alloc sum value exceeds the contract balance.
        if new_contract_allocs_sum_value_in_satoshis > contract_balance_in_satoshis {
            return Err(CMShadowDownAllError::AllocsSumExceedsTheContractBalance(
                contract_id,
                new_contract_allocs_sum_value_in_satoshis,
                contract_balance_in_satoshis,
            ));
        }

        // 7 Get the mutable ephemeral shadow space from the delta.
        let mut_epheremal_shadow_space = self
            .get_mut_ephemeral_contract_shadow_space(contract_id)
            .ok_or(CMShadowDownAllError::UnableToGetMutEphemeralShadowSpace(
                contract_id,
            ))?;

        // 8 Update the allocs_sum immediately (for validation in subsequent operations).
        mut_epheremal_shadow_space.update_allocs_sum(new_contract_allocs_sum_value_in_satoshis);

        // 9 Accumulate the deferred proportional change (negative value for down_all).
        mut_epheremal_shadow_space
            .add_deferred_proportional_change(-(down_value_in_satoshis as i64));

        // 10 Get the number of affected accounts (for return value).
        // 10.1 Count accounts that are not ephemerally deallocated.
        let num_affected_accounts = mut_epheremal_shadow_space.allocs.len() as u64;

        // 11 Return the number of affected accounts.
        Ok(num_affected_accounts)
    }

    /// Returns the list of accounts whose coin balances or allocations are changed in one way or another.
    pub fn get_coingap_accounts_list(&self) -> Vec<AccountKey> {
        self.delta.coingap_accounts_list()
    }

    /// Reverts the epheremal changes associated with the last execution.
    pub fn rollback_last(&mut self) {
        // Restore the ephemeral states from the backup.
        self.restore_delta();
    }

    /// Applies all epheremal changes from the delta into the permanent in-memory & on-disk.
    pub fn apply_changes(&mut self) -> Result<(), CMApplyChangesError> {
        // 1 Register new accounts in-memory and on-disk.
        for (account_key, initial_account_balance) in self.delta.new_accounts_to_register.iter() {
            // 1.1 A fresh new account has a zero allocs sum value.
            let initial_account_allocs_sum_value_in_sati_satoshis: u128 = 0;

            // 1.2 On-disk insertion.
            {
                // 1.2.1 Open on-disk accounts tree.
                let tree = self.on_disk_accounts.open_tree(account_key).map_err(|e| {
                    CMApplyChangesError::AccountApplyChangesError(
                        CMAccountApplyChangesError::OpenTreeError(*account_key, e),
                    )
                })?;

                // 1.2.2 Insert the account balance on-disk.
                {
                    tree.insert(
                        ACCOUNT_BALANCE_SPECIAL_DB_KEY,
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

                // 1.2.3 Insert the account shadow allocs value sum on-disk.
                {
                    tree.insert(
                        ACCOUNT_ALLOCS_SUM_SPECIAL_DB_KEY,
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

            // 1.3 In-memory insertion.
            {
                // 1.3.1 Construct the fresh new account body.
                let fresh_new_account_body = CMAccountBody::new(
                    *initial_account_balance,
                    initial_account_allocs_sum_value_in_sati_satoshis,
                );

                // 1.3.2 Insert the account balance into the in-memory list.
                // 1.3.3 Register the account in-memory with zero balance.
                self.in_memory_accounts
                    .insert(*account_key, fresh_new_account_body);
            }
        }

        // 2 Register new contracts in-memory and on-disk.
        for (contract_id, initial_contract_balance) in self.delta.new_contracts_to_register.iter() {
            // 2.1 A fresh new contract has a zero allocs sum value.
            let initial_contract_allocs_sum_value_in_satoshis: u64 = 0;

            // 2.2 On-disk insertion.
            {
                // 2.2.1 Open tree
                let tree = self.on_disk_contracts.open_tree(contract_id).map_err(|e| {
                    CMApplyChangesError::ContractApplyChangesError(
                        CMContractApplyChangesError::OpenTreeError(*contract_id, e),
                    )
                })?;

                // 2.2.2 Insert the contract balance on-disk.
                tree.insert(
                    CONTRACT_BALANCE_SPECIAL_DB_KEY,
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

                // 2.2.3 Insert the contract allocs sum value on-disk.
                tree.insert(
                    CONTRACT_ALLOCS_SUM_SPECIAL_DB_KEY,
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

            // 2.3 In-memory insertion.
            {
                // 2.3.1 Construct the fresh new shadow space.
                let fresh_new_shadow_space = ShadowSpace::fresh_new();

                // 2.3.2 Construct the fresh new contract body.
                let fresh_new_contract_body =
                    CMContractBody::new(*initial_contract_balance, fresh_new_shadow_space);

                // 2.3.3 Insert the contract body into the in-memory list.
                // 2.3.4 Register the contract in-memory.
                self.in_memory_contracts
                    .insert(*contract_id, fresh_new_contract_body);
            }
        }

        // 3 Save account balances.
        for (account_key, ephemeral_account_balance) in self.delta.updated_account_balances.iter() {
            // 3.1 On-disk insertion.
            {
                // 3.1.1 Open tree.
                let tree = self.on_disk_accounts.open_tree(account_key).map_err(|e| {
                    CMApplyChangesError::AccountApplyChangesError(
                        CMAccountApplyChangesError::OpenTreeError(*account_key, e),
                    )
                })?;

                // 3.1.2 Update the account balance on-disk.
                tree.insert(
                    ACCOUNT_BALANCE_SPECIAL_DB_KEY,
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

            // 3.2 In-memory insertion.
            {
                // 3.2.1 Get the mutable permanent account body from the permanent states.
                let mut_permanent_account_body = self
                    .in_memory_accounts
                    .get_mut(account_key)
                    .ok_or(CMApplyChangesError::AccountApplyChangesError(
                        CMAccountApplyChangesError::UnableToGetPermanentAccountBody(*account_key),
                    ))?;

                // 3.2.2 Update the account balance in-memory.
                mut_permanent_account_body.update_balance(*ephemeral_account_balance);
            }
        }

        // 4 Save contract balances.
        for (contract_id, ephemeral_contract_balance) in self.delta.updated_contract_balances.iter()
        {
            // 4.1 On-disk insertion.
            {
                // Open tree.
                let tree = self.on_disk_contracts.open_tree(contract_id).map_err(|e| {
                    CMApplyChangesError::ContractApplyChangesError(
                        CMContractApplyChangesError::OpenTreeError(*contract_id, e),
                    )
                })?;

                // Update the contract balance on-disk.
                tree.insert(
                    CONTRACT_BALANCE_SPECIAL_DB_KEY,
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

            // 4.2 In-memory insertion.
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
        }

        // 5 Apply deferred proportional changes (shadow_up_all/down_all) to shadow spaces and update delta.
        // 5.0 Track cumulative account global shadow allocs sum updates during iteration (to apply after the loop to avoid borrowing issues).
        // Use HashMap to track cumulative changes so each contract sees updates from previous contracts in the same loop.
        let mut account_global_shadow_allocs_sum_updates: std::collections::HashMap<
            AccountKey,
            SatiSatoshiAmount,
        > = std::collections::HashMap::new();

        for (_contract_id, ephemeral_shadow_space_mut) in
            self.delta.updated_shadow_spaces.iter_mut()
        {
            // 5.1 Check if there's a deferred proportional change to apply.
            let deferred_change_in_satoshis = ephemeral_shadow_space_mut.shadow_up_all_down_alls;

            if deferred_change_in_satoshis != 0 {
                // 5.1.1 Calculate the base allocs_sum (before deferred changes).
                let current_allocs_sum_in_satoshis = ephemeral_shadow_space_mut.allocs_sum;
                let base_allocs_sum_in_satoshis =
                    (current_allocs_sum_in_satoshis as i64 - deferred_change_in_satoshis) as u64;

                // 5.1.2 Check if base allocs sum is zero (should not happen if we validated correctly).
                if base_allocs_sum_in_satoshis != 0 {
                    // 5.1.3 Convert values to sati-satoshis for calculation.
                    let base_allocs_sum_in_sati_satoshis =
                        (base_allocs_sum_in_satoshis as u128) * ONE_SATOSHI_IN_SATI_SATOSHIS;
                    let deferred_change_in_sati_satoshis =
                        (deferred_change_in_satoshis.abs() as u128) * ONE_SATOSHI_IN_SATI_SATOSHIS;

                    // 5.1.4 Iterate over all allocations and apply proportional changes.
                    let allocs_copy: Vec<(AccountKey, SatiSatoshiAmount)> =
                        ephemeral_shadow_space_mut
                            .allocs
                            .iter()
                            .map(|(k, v)| (*k, *v))
                            .collect();

                    for (account_key, base_alloc_value_in_sati_satoshis) in allocs_copy.iter() {
                        // 5.1.4.1 Calculate the proportional change for this account.
                        let individual_change_in_sati_satoshis = if deferred_change_in_satoshis > 0
                        {
                            // Up_all: proportional increase
                            (base_alloc_value_in_sati_satoshis * deferred_change_in_sati_satoshis)
                                / base_allocs_sum_in_sati_satoshis
                        } else {
                            // Down_all: proportional decrease
                            let individual_down = (base_alloc_value_in_sati_satoshis
                                * deferred_change_in_sati_satoshis)
                                / base_allocs_sum_in_sati_satoshis;
                            // Ensure we don't go below zero.
                            individual_down.min(*base_alloc_value_in_sati_satoshis)
                        };

                        // 5.1.4.2 Calculate the new alloc value.
                        let new_alloc_value_in_sati_satoshis = if deferred_change_in_satoshis > 0 {
                            base_alloc_value_in_sati_satoshis + individual_change_in_sati_satoshis
                        } else {
                            base_alloc_value_in_sati_satoshis
                                .saturating_sub(individual_change_in_sati_satoshis)
                        };

                        // 5.1.4.3 Update the allocation value in the shadow space.
                        ephemeral_shadow_space_mut.insert_update_alloc(
                            account_key.to_owned(),
                            new_alloc_value_in_sati_satoshis,
                        );

                        // 5.1.4.4 Track the change for account global shadow allocs sum update.
                        if individual_change_in_sati_satoshis > 0 {
                            // Calculate the change amount.
                            let change = if deferred_change_in_satoshis > 0 {
                                individual_change_in_sati_satoshis as i128
                            } else {
                                -(individual_change_in_sati_satoshis as i128)
                            };

                            // Get current value, checking cumulative updates first (from previous contracts in this loop),
                            // then delta (from before this loop), then permanent state.
                            // This ensures changes are cumulative across contracts in the same loop iteration.
                            let current_account_global_shadow_allocs_sum =
                                account_global_shadow_allocs_sum_updates
                                    .get(account_key)
                                    .copied()
                                    .or_else(|| {
                                        self.delta
                                            .updated_global_shadow_allocs_sums
                                            .get(account_key)
                                            .copied()
                                    })
                                    .or_else(|| {
                                        self.in_memory_accounts
                                            .get(account_key)
                                            .map(|body| body.global_shadow_allocs_sum)
                                    })
                                    .unwrap_or(0);

                            let new_account_global_shadow_allocs_sum = if change > 0 {
                                current_account_global_shadow_allocs_sum
                                    .checked_add(change as u128)
                                    .expect("Account global shadow allocs sum overflow on deferred proportional change")
                            } else {
                                current_account_global_shadow_allocs_sum
                                    .checked_sub((-change) as u128)
                                    .expect("Account global shadow allocs sum underflow on deferred proportional change")
                            };

                            // Store cumulative update (will overwrite if same account appears again, with the cumulative value).
                            account_global_shadow_allocs_sum_updates.insert(
                                account_key.to_owned(),
                                new_account_global_shadow_allocs_sum,
                            );
                        }
                    }
                }

                // 5.1.5 Clear the deferred proportional change.
                ephemeral_shadow_space_mut.clear_deferred_proportional_change();
            }
        }

        // 5.2 Apply all account global shadow allocs sum updates to delta (outside the borrow of updated_shadow_spaces).
        for (account_key, new_value) in account_global_shadow_allocs_sum_updates {
            self.delta
                .epheremally_update_account_global_shadow_allocs_sum(account_key, new_value);
        }

        // 6 Save account's updated global shadow allocs sum values.
        // NOTE: This also automatically handles new allocations.
        for (account_key, ephemeral_account_global_shadow_allocs_sum) in
            self.delta.updated_global_shadow_allocs_sums.iter()
        {
            // 5.1 On-disk insertion.
            {
                // Open tree.
                let tree = self.on_disk_accounts.open_tree(account_key).map_err(|e| {
                    CMApplyChangesError::AccountApplyChangesError(
                        CMAccountApplyChangesError::OpenTreeError(*account_key, e),
                    )
                })?;

                // Update the global shadow allocs sum on-disk.
                tree.insert(
                    ACCOUNT_ALLOCS_SUM_SPECIAL_DB_KEY,
                    ephemeral_account_global_shadow_allocs_sum
                        .to_le_bytes()
                        .to_vec(),
                )
                .map_err(|e| {
                    CMApplyChangesError::AccountApplyChangesError(
                        CMAccountApplyChangesError::ShadowAllocsSumValueOnDiskInsertionError(
                            account_key.to_owned(),
                            *ephemeral_account_global_shadow_allocs_sum,
                            e,
                        ),
                    )
                })?;
            }

            // 5.2 In-memory insertion.
            {
                // Get the mutable permanent account body.
                let mut_permanent_account_body = self
                    .in_memory_accounts
                    .get_mut(account_key)
                    .ok_or(CMApplyChangesError::AccountApplyChangesError(
                        CMAccountApplyChangesError::UnableToGetPermanentAccountBody(*account_key),
                    ))?;

                // Update the global shadow allocs sum in-memory.
                mut_permanent_account_body
                    .update_global_shadow_allocs_sum(*ephemeral_account_global_shadow_allocs_sum);
            }
        }

        // 7 Save contract's updated shadow spaces.
        for (contract_id, ephemeral_shadow_space) in self.delta.updated_shadow_spaces.iter() {
            // Get the final shadow allocs sum value.
            let final_shadow_allocs_sum_value = ephemeral_shadow_space.allocs_sum;

            // 7.1 On-disk insertion.
            {
                // Open tree.
                let tree = self.on_disk_contracts.open_tree(contract_id).map_err(|e| {
                    CMApplyChangesError::ContractApplyChangesError(
                        CMContractApplyChangesError::OpenTreeError(*contract_id, e),
                    )
                })?;

                // Update alloc values one-by-one on-disk.
                for (shadow_account_key, shadow_alloc_value) in ephemeral_shadow_space.allocs.iter()
                {
                    // Update the shadow alloc value on-disk.
                    tree.insert(
                        shadow_account_key.to_vec(),
                        shadow_alloc_value.to_le_bytes().to_vec(),
                    )
                    .map_err(|e| {
                        CMApplyChangesError::ContractApplyChangesError(
                            CMContractApplyChangesError::ShadowAllocValueOnDiskInsertionError(
                                *contract_id,
                                *shadow_account_key,
                                *shadow_alloc_value,
                                e,
                            ),
                        )
                    })?;
                }

                // Update the allocs sum value on-disk.
                tree.insert(
                    CONTRACT_ALLOCS_SUM_SPECIAL_DB_KEY,
                    final_shadow_allocs_sum_value.to_le_bytes().to_vec(),
                )
                .map_err(|e| {
                    CMApplyChangesError::ContractApplyChangesError(
                        CMContractApplyChangesError::AllocsSumValueOnDiskInsertionError(
                            *contract_id,
                            final_shadow_allocs_sum_value,
                            e,
                        ),
                    )
                })?;
            }

            // 7.2 In-memory insertion.
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
        }

        // 8 Handle deallocations.
        {
            for (contract_id, ephemeral_dealloc_list) in self.delta.deallocs_list.iter() {
                // 7.1 On-disk deletion.
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

                // 7.2 In-memory deletion.
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
                            .shadow_space
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
            }
        }

        // 9 Return the result.
        Ok(())
    }

    /// Returns the account's overall flame sum value (owned and owed value sum) in satoshis.
    ///
    /// NOTE: This is a post-apply-changes function, called by the FlameManager.
    pub fn get_account_target_flame_value_in_satoshis(
        &self,
        account_key: AccountKey,
    ) -> Option<u64> {
        // 1 Get the account's balance in satoshis.
        let account_balance_in_satoshis: u64 = self.get_account_balance(account_key)?;

        // 2 Get the account's global shadow allocs sum in satoshis.
        // NOTE: get_account_target_flame_value_in_satoshis is called post-apply-changes, so we can safely use the base value here.
        let account_global_shadow_allocs_sum_in_satoshis: u64 =
            self.get_account_global_shadow_allocs_sum_in_satoshis_base(account_key)?;

        // 3 Calculate the account's overall owned and owed value in satoshis.
        let account_overall_owned_and_owed_value_in_satoshis: u64 =
            account_balance_in_satoshis + account_global_shadow_allocs_sum_in_satoshis;

        // 4 Return the result.
        Some(account_overall_owned_and_owed_value_in_satoshis)
    }

    /// Clears all epheremal changes from the delta.
    pub fn flush_delta(&mut self) {
        // Clear the ephemeral states.
        self.delta.flush();

        // Clear the ephemeral states backup.
        self.backup_of_delta.flush();
    }

    // Return as json the whole state of the coin manager.
    pub fn json(&self) -> Value {
        // 1 Construct the coin manager JSON object.
        let mut obj = Map::new();

        // 2 Insert account bodies.
        obj.insert(
            "accounts".to_string(),
            Value::Object(
                self.in_memory_accounts
                    .iter()
                    .map(|(account_key, account_body)| {
                        (hex::encode(account_key), account_body.json())
                    })
                    .collect(),
            ),
        );

        // 3 Insert contract bodies.
        obj.insert(
            "contracts".to_string(),
            Value::Object(
                self.in_memory_contracts
                    .iter()
                    .map(|(contract_id, contract_body)| {
                        (hex::encode(contract_id), contract_body.json())
                    })
                    .collect(),
            ),
        );

        // 4 Return the coin manager JSON object.
        Value::Object(obj)
    }
}

/// Erases the coin manager by db paths.
pub fn erase_coin_manager(chain: Chain) {
    // Accounts db path.
    let accounts_db_path = format!("storage/{}/coins/accounts", chain.to_string());

    // Erase the accounts db path.
    let _ = std::fs::remove_dir_all(accounts_db_path);

    // Contracts db path.
    let contracts_db_path = format!("storage/{}/coins/contracts", chain.to_string());

    // Erase the contracts db path.
    let _ = std::fs::remove_dir_all(contracts_db_path);
}
