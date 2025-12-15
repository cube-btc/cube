use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::flame_manager::delta::delta::FMDelta;
use crate::inscriptive::flame_manager::errors::apply_changes_error::FMApplyChangesError;
use crate::inscriptive::flame_manager::errors::construction_error::FMConstructionError;
use crate::inscriptive::flame_manager::errors::register_account_error::FMRegisterAccountError;
use crate::inscriptive::flame_manager::errors::update_account_flame_config_error::FMUpdateAccountFlameConfigError;
use crate::inscriptive::flame_manager::flame::flame::Flame;
use crate::inscriptive::flame_manager::flame_config::flame_config::FMAccountFlameConfig;
use crate::operative::Chain;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Account key.
type AccountKey = [u8; 32];

/// Projector height.
type ProjectorHeight = u64;

/// Flame index.
pub type FlameIndex = u32;

/// Special db key for the account flame config (0x00..).
const ACCOUNT_FLAME_CONFIG_SPECIAL_DB_KEY: [u8; 1] = [0x00; 1];

/// Flame manager.
#[allow(dead_code)]
pub struct FlameManager {
    // In-memory account flame configs.
    in_memory_account_flame_configs: HashMap<AccountKey, FMAccountFlameConfig>,

    // In-memory account flame sets.
    in_memory_account_flame_sets:
        HashMap<AccountKey, HashMap<ProjectorHeight, Vec<(FlameIndex, Flame)>>>,

    // In-memory global flame set.
    in_memory_global_flame_set: HashMap<ProjectorHeight, Vec<(FlameIndex, Flame)>>,

    // On-disk accounts database.
    on_disk_accounts: sled::Db,

    // On-disk global flame set database.
    on_disk_global_flame_set: sled::Db,

    // State differences to be applied.
    delta: FMDelta,

    // Backup of state differences in case of rollback.
    backup_of_delta: FMDelta,
}

/// Guarded 'FlameManager'.
#[allow(non_camel_case_types)]
pub type FLAME_MANAGER = Arc<Mutex<FlameManager>>;

impl FlameManager {
    /// Constructs a fresh new 'FlameManager'.
    pub fn new(chain: Chain) -> Result<FLAME_MANAGER, FMConstructionError> {
        // 1 Open the accounts db.
        let accounts_db_path = format!("storage/{}/flames/accounts", chain.to_string());
        let accounts_db =
            sled::open(accounts_db_path).map_err(FMConstructionError::AccountsDBOpenError)?;

        // 2 Open the global flame set db.
        let global_flame_set_db_path = format!("storage/{}/flames/global", chain.to_string());
        let global_flame_set_db = sled::open(global_flame_set_db_path)
            .map_err(FMConstructionError::GlobalFlameSetDBOpenError)?;

        // 3 Initialize the in-memory account flame configs and sets.
        let mut in_memory_account_flame_configs =
            HashMap::<AccountKey, FMAccountFlameConfig>::new();
        let mut in_memory_account_flame_sets =
            HashMap::<AccountKey, HashMap<ProjectorHeight, Vec<(FlameIndex, Flame)>>>::new();

        // 4 Collect account flame configs and sets from the accounts database.
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
            let tree = accounts_db
                .open_tree(tree_name)
                .map_err(|e| FMConstructionError::AccountsTreeOpenError(account_key, e))?;

            // 4.3 Initialize the account flame config and flames grouped by rollup height.
            let mut account_flame_config: Option<FMAccountFlameConfig> = None;
            let mut account_flames_by_height: HashMap<ProjectorHeight, Vec<(FlameIndex, Flame)>> =
                HashMap::new();

            // 4.4 Iterate over all items in the tree.
            for item in tree.iter() {
                // 4.4.1 Get the key and value.
                let (key, value) = match item {
                    Ok((k, v)) => (k, v),
                    Err(e) => {
                        return Err(FMConstructionError::AccountsTreeIterError(account_key, e));
                    }
                };

                // 4.4.2 Check if this is the special config key or a flame index key.
                if key.as_ref().len() == 1 {
                    let key_byte: [u8; 1] = key.as_ref().try_into().map_err(|_| {
                        FMConstructionError::UnableToDeserializeAccountDbKeyByteFromTreeKey(
                            account_key,
                            key.to_vec(),
                        )
                    })?;

                    // 0x00 key byte represents the account flame config.
                    if key_byte == ACCOUNT_FLAME_CONFIG_SPECIAL_DB_KEY {
                        if value.as_ref().len() > 0 {
                            // Deserialize the flame config from bytes.
                            let flame_config_deserialized = FMAccountFlameConfig::from_db_value_bytes(value.as_ref())
                                .ok_or(FMConstructionError::UnableToDeserializeAccountFlameConfigBytesFromTreeValue(
                                    account_key,
                                    value.to_vec(),
                                ))?;

                            // Update the account flame config.
                            account_flame_config = Some(flame_config_deserialized);
                        }
                        continue;
                    }
                }

                // 4.4.3 Convert the tree key to 12 bytes: 8-byte height + 4-byte index.
                if key.as_ref().len() != 12 {
                    return Err(FMConstructionError::InvalidAccountDbKeyByte(
                        account_key,
                        key.to_vec(),
                    ));
                }

                let rollup_height_bytes: [u8; 8] = key.as_ref()[0..8].try_into().unwrap();
                let rollup_height = u64::from_le_bytes(rollup_height_bytes);

                let flame_index_bytes: [u8; 4] = key.as_ref()[8..12].try_into().unwrap();
                let flame_index = u32::from_le_bytes(flame_index_bytes);

                // 4.4.4 Deserialize the value: literal flame bytes (no prefix).
                let flame = Flame::from_bytes(value.as_ref()).ok_or(
                    FMConstructionError::UnableToDeserializeAccountFlameSetBytesFromTreeValue(
                        account_key,
                        value.to_vec(),
                    ),
                )?;

                // 4.4.5 Store the flame grouped by rollup height.
                account_flames_by_height
                    .entry(rollup_height)
                    .or_insert_with(Vec::new)
                    .push((flame_index, flame));
            }

            // 4.5 Insert the account flame config if it exists.
            if let Some(config) = account_flame_config {
                in_memory_account_flame_configs.insert(account_key, config);
            }

            // 4.6 Sort flames by index within each rollup height and insert.
            if !account_flames_by_height.is_empty() {
                for flames in account_flames_by_height.values_mut() {
                    flames.sort_by_key(|(flame_index, _)| *flame_index);
                }
                in_memory_account_flame_sets.insert(account_key, account_flames_by_height);
            }
        }

        // 5 Initialize the in-memory projected flames.
        let mut in_memory_global_flame_set =
            HashMap::<ProjectorHeight, Vec<(FlameIndex, Flame)>>::new();

        // 6 Collect projected flames from the projected flames database.
        for tree_name in global_flame_set_db.tree_names() {
            // 6.1 Deserialize rollup height bytes from tree name.
            let rollup_height: [u8; 8] = match tree_name.as_ref().try_into() {
                Ok(height) => height,
                Err(_) => {
                    // Tree name is probably '__sled__default'. Skip it.
                    continue;
                }
            };
            let rollup_height = u64::from_le_bytes(rollup_height);

            // 6.2 Open the tree.
            let tree = global_flame_set_db
                .open_tree(tree_name)
                .map_err(|e| FMConstructionError::GlobalFlameSetTreeOpenError(rollup_height, e))?;

            // 6.3 Initialize the list of flames with their indices for this rollup height.
            let mut flames_with_indices: Vec<(FlameIndex, Flame)> = Vec::new();

            // 6.4 Iterate over all items in the tree.
            for item in tree.iter() {
                // 6.4.1 Get the key and value.
                let (key, value) = match item {
                    Ok((k, v)) => (k, v),
                    Err(e) => {
                        return Err(FMConstructionError::GlobalFlameSetTreeIterError(
                            rollup_height,
                            e,
                        ));
                    }
                };

                // 6.4.2 Convert the tree key to a 4-byte index.
                let flame_index_bytes: [u8; 4] = match key.as_ref().try_into() {
                    Ok(idx) => idx,
                    Err(_) => {
                        return Err(
                            FMConstructionError::UnableToDeserializeFlameIndexBytesFromTreeKey(
                                rollup_height,
                                value.to_vec(),
                            ),
                        );
                    }
                };
                let flame_index = u32::from_le_bytes(flame_index_bytes);

                // 6.4.3 Deserialize the value: literal flame bytes (no prefix, index is in the key).
                let flame = Flame::from_bytes(value.as_ref()).ok_or(
                    FMConstructionError::UnableToDeserializeFlameBytesFromTreeValue(
                        rollup_height,
                        value.to_vec(),
                    ),
                )?;

                // 6.4.4 Store the flame with its index.
                flames_with_indices.push((flame_index, flame));
            }

            // 6.5 Sort flames by index and insert for this rollup height.
            if !flames_with_indices.is_empty() {
                flames_with_indices.sort_by_key(|(flame_index, _)| *flame_index);
                in_memory_global_flame_set.insert(rollup_height, flames_with_indices);
            }
        }

        // 7 Construct the flame manager.
        let flame_manager = FlameManager {
            in_memory_account_flame_configs,
            in_memory_account_flame_sets,
            in_memory_global_flame_set,
            on_disk_accounts: accounts_db,
            on_disk_global_flame_set: global_flame_set_db,
            delta: FMDelta::fresh_new(),
            backup_of_delta: FMDelta::fresh_new(),
        };

        // 8 Guard the flame manager.
        let guarded_flame_manager = Arc::new(Mutex::new(flame_manager));

        // 9 Return the guarded flame manager.
        Ok(guarded_flame_manager)
    }

    /// Clones the delta into the backup.
    fn backup_delta(&mut self) {
        self.backup_of_delta = self.delta.clone();
    }

    /// Restores the delta from the backup.
    fn restore_delta(&mut self) {
        self.delta = self.backup_of_delta.clone();
    }

    /// Prepares the registery manager prior to each execution.
    ///
    /// NOTE: Used by the Engine.
    pub fn pre_execution(&mut self) {
        self.backup_delta();
    }

    /// Checks if an account is permanently registered.
    pub fn is_account_registered(&self, account_key: AccountKey) -> bool {
        self.in_memory_account_flame_configs
            .contains_key(&account_key)
    }

    /// Returns the flame config for a given account.
    pub fn get_account_flame_config(
        &self,
        account_key: AccountKey,
    ) -> Option<FMAccountFlameConfig> {
        self.in_memory_account_flame_configs
            .get(&account_key)
            .cloned()
    }

    /// Returns the flame set for a given account.
    pub fn get_account_flame_set(
        &self,
        account_key: AccountKey,
    ) -> Option<HashMap<ProjectorHeight, Vec<(FlameIndex, Flame)>>> {
        // 1 Check if the account is permanently registered.
        match self.in_memory_account_flame_sets.get(&account_key) {
            // 1.a The account has a flame set.
            Some(flame_set) => Some(flame_set.to_owned()),

            // 1.b The account does not have a flame set.
            None => None,
        }
    }

    /// Epheremally registers an account.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn register_account(
        &mut self,
        account_key: AccountKey,
        flame_config: Option<FMAccountFlameConfig>,
    ) -> Result<(), FMRegisterAccountError> {
        // 1 Check if the account is already permanently registered.
        if self.is_account_registered(account_key) {
            return Err(FMRegisterAccountError::AccountIsAlreadyPermanentlyRegistered(account_key));
        }

        // 2 Epheremally register the account in the delta.
        if !self
            .delta
            .epheremally_register_account(account_key, flame_config)
        {
            // 2.1 Return an error if the account has just been epheremally registered in the delta.
            return Err(
                FMRegisterAccountError::AccountHasJustBeenEphemerallyRegistered(account_key),
            );
        }

        // 3 Return the result.
        Ok(())
    }

    /// Epheremally updates an account's flame config.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn epheremally_update_account_flame_config(
        &mut self,
        account_key: AccountKey,
        flame_config: FMAccountFlameConfig,
    ) -> Result<Option<FMAccountFlameConfig>, FMUpdateAccountFlameConfigError> {
        // 1 Check if the account is permanently registered.
        if !self.is_account_registered(account_key) {
            return Err(FMUpdateAccountFlameConfigError::AccountIsNotRegistered(
                account_key,
            ));
        }

        // 2 Check if the flame config has already been epheremally updated in the delta.
        if self.delta.updated_flame_configs.contains_key(&account_key) {
            return Err(
                FMUpdateAccountFlameConfigError::AccountFlameConfigHasAlreadyEpheremallyUpdated(
                    account_key,
                ),
            );
        }

        // 3 Epheremally update the account's flame config in the delta.
        self.delta
            .epheremally_update_account_flame_config(account_key, flame_config);

        // 4 Get the previous flame config if there is one.
        let previous_flame_config = self
            .in_memory_account_flame_configs
            .get(&account_key)
            .cloned();

        // 5 Return the result.
        Ok(previous_flame_config)
    }

    /// Reverts the epheremal changes associated with the last execution.
    ///
    /// NOTE: Used by the Engine.
    pub fn rollback_last(&mut self) {
        // Restore the epheremal changes from the backup.
        self.restore_delta();
    }

    /// Clears all epheremal changes from the delta.
    ///
    /// NOTE: Used by the Engine.
    pub fn flush_delta(&mut self) {
        // Clear the epheremal changes from the delta.
        self.delta.flush();

        // Clear the epheremal changes from the backup.
        self.backup_of_delta.flush();
    }

    pub async fn apply_changes(
        &mut self,
        coin_manager: &COIN_MANAGER,
        new_projector_height: ProjectorHeight,
        projector_expiry_height: ProjectorHeight,
    ) -> Result<(), FMApplyChangesError> {
        // TODO:Register new accounts

        // TODO: Update account flame configs

        // 2 Get the overall affected accounts list (expired flames accounts + affected coin manager accounts).
        let overall_affected_accounts: Vec<AccountKey> = {
            // 2.1 Initialize the overall affected accounts list.
            let mut overall_affected_accounts: Vec<AccountKey> = Vec::new();

            // 2.2 Get affected coin manager accounts.
            // 1 Get the affected coin manager accounts list.
            let affected_coin_manager_accounts: Vec<AccountKey> = {
                // 1.1 Lock the coin manager.
                let _coin_manager = coin_manager.lock().await;

                // 1.2 Get the affected coin manager accounts list.
                _coin_manager.get_affected_accounts_list()
            };

            // 2.3 Extend the overall affected accounts list with the affected coin manager accounts.
            overall_affected_accounts.extend(affected_coin_manager_accounts);

            // 2.4 Get affected expired flames accounts.
            let affected_expired_flames_accounts: Vec<AccountKey> = {
                // 2.4.1 Initialize the affected expired flames accounts list.
                let mut affected_expired_flames_accounts: Vec<AccountKey> = Vec::new();

                // 2.4.2 Iterate over all in-memory account flame sets.
                for (account_key, account_flame_set) in self.in_memory_account_flame_sets.iter() {
                    // 2.4.2.1 Iterate over all rollup heights in the account flame set.
                    'inner_loop: for (projector_height, _) in account_flame_set.iter() {
                        // 2.4.2.1.1 Check if the rollup height is below or equal to the projector rollup height.
                        if *projector_height <= projector_expiry_height {
                            // 2.4.2.1.1.1 Insert the account key into the affected expired flames accounts list.
                            if !affected_expired_flames_accounts.contains(account_key) {
                                affected_expired_flames_accounts.push(account_key.to_owned());
                            }

                            // 2.4.2.1.1.2 Break the inner loop.
                            break 'inner_loop;
                        }
                    }
                }

                // 2.4.3 Return the affected expired flames accounts list.
                affected_expired_flames_accounts
            };

            // 2.5 Extend the overall affected accounts list with the affected expired flames accounts.
            for account_key in affected_expired_flames_accounts {
                // 2.5.1 Insert only if not already present.
                if !overall_affected_accounts.contains(&account_key) {
                    overall_affected_accounts.push(account_key);
                }
            }

            // 2.6 Return the overall affected accounts list.
            overall_affected_accounts
        };

        let mut new_flames_to_insert: HashMap<AccountKey, Vec<Flame>> = HashMap::new();

        // 3 Collect new flames to insert and prune expired flames in the meantime.
        {
            // 3.1 Lock the coin manager.
            let _coin_manager = coin_manager.lock().await;

            // 3.1 Iterate over all affected accounts.
            'affected_accounts_loop: for account_key in overall_affected_accounts {
                // Get the account flame config.
                let account_flame_config: FMAccountFlameConfig = {
                    match self
                        .in_memory_account_flame_configs
                        .get(&account_key)
                        .cloned()
                    {
                        Some(flame_config) => flame_config,
                        None => {
                            // If flame config not set, continue to the next affected account.
                            continue 'affected_accounts_loop;
                        }
                    }
                };

                // 3.1.1 Open the tree for the account.
                let tree = self
                    .on_disk_accounts
                    .open_tree(account_key)
                    .map_err(|e| FMApplyChangesError::AccountTreeOpenError(account_key, e))?;

                // 3.1.2 Get the in-memory mutable account flame set.
                let account_flame_set_mut = self
                    .in_memory_account_flame_sets
                    .get_mut(&account_key)
                    .expect("This should never happen.");

                // 3.1.3 Initialize the list of pruned flame db keys.
                let mut pruned_flame_db_keys: Vec<[u8; 12]> = Vec::new();

                // 3.1.3 Iterate over all rollup heights in the account flame set.
                for (projector_height, flames) in account_flame_set_mut.clone().iter() {
                    // 3.1.3.1 Check if the rollup height is below or equal to the projector rollup height.
                    if *projector_height <= projector_expiry_height {
                        // 3.1.3.1.1 Iterate over all flames in the rollup height.
                        for (flame_index, _) in flames.iter() {
                            // 3.1.3.1.1.1 Convert the rollup height and flame index to a 12 byte db key.
                            let flame_db_key: [u8; 12] = {
                                // 3.1.3.1.1.1.1 Initialize the flame db key.
                                let mut flame_db_key: [u8; 12] = [0; 12];

                                // 3.1.3.1.1.1.2 Copy the rollup height bytes to the flame db key.
                                flame_db_key[0..8].copy_from_slice(&projector_height.to_le_bytes());

                                // 3.1.3.1.1.1.3 Copy the flame index bytes to the flame db key.
                                flame_db_key[8..12].copy_from_slice(&flame_index.to_le_bytes());

                                // 3.1.3.1.1.1.4 Return the flame db key.
                                flame_db_key
                            };

                            // 3.1.3.1.1.2 Insert the flame db key into the list of pruned flame db keys.
                            pruned_flame_db_keys.push(flame_db_key);
                        }

                        // 3.1.3.1.2 Prune expired flames from the in-memory account flame set.
                        account_flame_set_mut.remove(projector_height);
                    }
                }

                // 3.1.5 Prune expired flames from the on-disk account flame set.
                for flame_db_key_to_prune in pruned_flame_db_keys {
                    // 3.1.5.1 Remove the flame from the tree.
                    tree.remove(flame_db_key_to_prune).map_err(|e| {
                        FMApplyChangesError::AccountRemoveFlameFromDiskTreeError(
                            account_key,
                            flame_db_key_to_prune,
                            e,
                        )
                    })?;
                }

                // Get the target flame value for the account.
                let account_target_flame_value_in_satoshis: u64 = {
                    _coin_manager
                        .get_account_target_flame_value_in_satoshis(account_key)
                        .ok_or(
                            FMApplyChangesError::AccountTargetFlameValueCouldNotBeRetrieved(
                                account_key,
                            ),
                        )?
                };

                // Get the current flame set sum value.
                let account_current_flame_set_sum_value_in_satoshis: u64 = {
                    // 3.1.4.1 Initialize the current flame set sum value.
                    let mut account_current_flame_set_sum_value_in_satoshis: u64 = 0;

                    // 3.1.4.2 Iterate over all flames in the account flame set.
                    for (_, flames) in account_flame_set_mut.iter() {
                        // 3.1.4.2.1 Add the flame value to the current flame set sum value.
                        for (_, flame) in flames.iter() {
                            account_current_flame_set_sum_value_in_satoshis +=
                                flame.satoshi_amount();
                        }
                    }

                    // 3.1.4.3 Return the current flame set sum value.
                    account_current_flame_set_sum_value_in_satoshis
                };

                // Retrieve the flames to fund.
                let flames_to_fund: Vec<Flame> = account_flame_config
                    .retrieve_flames_to_fund(
                        account_target_flame_value_in_satoshis,
                        account_current_flame_set_sum_value_in_satoshis,
                    )
                    .unwrap_or_default();

                // Insert the flames to fund into the in-memory account flame set.
                new_flames_to_insert.insert(account_key, flames_to_fund);
            }
        }

        // Sort the new flames to insert.
        let sorted_new_flames_to_insert: HashMap<AccountKey, Vec<(FlameIndex, Flame)>> =
            sort_new_flames_to_insert(new_flames_to_insert);

        // Apply changes to the account flame sets (inserts-only, pruning is already done).
        for (account_key, flames) in sorted_new_flames_to_insert.iter() {
            // Insert in-memory.
            {
                // Get the mutable account flame set.
                let flame_set_mut = self
                    .in_memory_account_flame_sets
                    .get_mut(account_key)
                    .expect("This should never happen.");

                // Iterate and insert the new flames into the account flame set.
                flame_set_mut.insert(new_projector_height, flames.to_owned());
            }

            // Insert on-disk.
            {
                // Open the tree for the account.
                let tree = self.on_disk_accounts.open_tree(account_key).map_err(|e| {
                    FMApplyChangesError::AccountTreeOpenError(account_key.to_owned(), e)
                })?;

                // Iterate and insert the new flames into the on-disk account flame set.
                for (flame_index, flame) in flames.iter() {
                    // Convert the rollup height and flame index to a 12 byte db key.
                    let flame_db_key: [u8; 12] = {
                        let mut flame_db_key: [u8; 12] = [0; 12];
                        flame_db_key[0..8].copy_from_slice(&new_projector_height.to_le_bytes());
                        flame_db_key[8..12].copy_from_slice(&flame_index.to_le_bytes());
                        flame_db_key
                    };

                    // Insert the new flame into the on-disk account flame set.
                    tree.insert(flame_db_key, flame.to_bytes()).map_err(|e| {
                        FMApplyChangesError::AccountInsertFlameIntoDiskTreeError(
                            account_key.to_owned(),
                            flame_db_key,
                            e,
                        )
                    })?;
                }
            }
        }

        // Apply changes to the global flame set (inserts & pruning).
        {
            // Pruning.
            {
                // In-memory pruning.
                {
                    self.in_memory_global_flame_set
                        .retain(|projector_height, _| *projector_height > projector_expiry_height);
                }

                // On-disk pruning.
                {
                    for tree_name in self.on_disk_global_flame_set.tree_names() {
                        // Convert the tree name to a rollup height.
                        let rollup_height: [u8; 8] = match tree_name.as_ref().try_into() {
                            Ok(height) => height,
                            Err(_) => {
                                // Tree name is probably '__sled__default'. Skip it.
                                continue;
                            }
                        };

                        // Convert the rollup height bytes to a u64.
                        let rollup_height = u64::from_le_bytes(rollup_height);

                        // Check if the rollup height is below or equal to the projector rollup height.
                        if rollup_height <= projector_expiry_height {
                            // Drop the tree.
                            self.on_disk_global_flame_set
                                .drop_tree(tree_name)
                                .map_err(|e| {
                                    FMApplyChangesError::GlobalFlameSetTreeDropError(
                                        projector_expiry_height,
                                        e,
                                    )
                                })?;
                        }
                    }
                }
            }

            // Insertions.
            {
                // In-memory insertions.
                {
                    // Collect new flames to insert.
                    let new_flames_to_insert: Vec<(FlameIndex, Flame)> = {
                        // Initialize the new flames to insert list.
                        let mut new_flames_to_insert: Vec<(FlameIndex, Flame)> = Vec::new();

                        // Iterate over all sorted new flames to insert.
                        for (_, flames) in sorted_new_flames_to_insert.iter() {
                            for (flame_index, flame) in flames.iter() {
                                new_flames_to_insert.push((flame_index.to_owned(), flame.clone()));
                            }
                        }

                        // Return the new flames to insert list.
                        new_flames_to_insert
                    };

                    // Insert the new flames into the in-memory global flame set.
                    self.in_memory_global_flame_set
                        .insert(new_projector_height, new_flames_to_insert);
                }

                // On-disk insertions.
                {
                    // Open new tree for the global flame set.
                    let tree = self
                        .on_disk_global_flame_set
                        .open_tree(new_projector_height.to_string())
                        .map_err(|e| {
                            FMApplyChangesError::GlobalFlameSetTreeOpenError(
                                new_projector_height,
                                e,
                            )
                        })?;

                    // Iterate over sorted new flames to insert.
                    for (_, flames) in sorted_new_flames_to_insert.iter() {
                        // Iterate over all flames in the account.
                        for (flame_index, flame) in flames.iter() {
                            let db_key_bytes: [u8; 4] = flame_index.to_le_bytes();

                            let db_value_bytes = flame.to_bytes();

                            // Insert the new flame into the on-disk global flame set.
                            tree.insert(db_key_bytes, db_value_bytes).map_err(|e| {
                                FMApplyChangesError::GlobalFlameSetInsertFlameIntoDiskTreeError(
                                    new_projector_height,
                                    *flame_index,
                                    e,
                                )
                            })?;
                        }
                    }
                }
            }

            // Return Ok.
            Ok(())
        }
    }
}
fn sort_new_flames_to_insert(
    new_flames_to_insert: HashMap<AccountKey, Vec<Flame>>,
) -> HashMap<AccountKey, Vec<(FlameIndex, Flame)>> {
    // 1 Collect all (account_key, flame) pairs.
    let mut all_flames_with_accounts: Vec<(AccountKey, Flame)> = Vec::new();
    for (account_key, flames) in new_flames_to_insert.iter() {
        for flame in flames.iter() {
            all_flames_with_accounts.push((*account_key, flame.clone()));
        }
    }

    // 2 Sort by flame value (descending), then by account key (ascending lexicographically).
    all_flames_with_accounts.sort_by(|(account_key_a, flame_a), (account_key_b, flame_b)| {
        // 2.1 First, compare by flame value (descending - higher values first).
        match flame_b.satoshi_amount().cmp(&flame_a.satoshi_amount()) {
            std::cmp::Ordering::Equal => {
                // 2.2 If flame values are equal, compare by account key (ascending - smaller keys first).
                account_key_a.cmp(account_key_b)
            }
            other => other,
        }
    });

    // 3 Assign indices starting from 0 and group back by account.
    let mut result: HashMap<AccountKey, Vec<(FlameIndex, Flame)>> = HashMap::new();
    for (index, (account_key, flame)) in all_flames_with_accounts.iter().enumerate() {
        let flame_index = index as FlameIndex;
        result
            .entry(*account_key)
            .or_insert_with(Vec::new)
            .push((flame_index, flame.clone()));
    }

    // 4 Sort flames within each account by index (for consistency).
    for flames in result.values_mut() {
        flames.sort_by_key(|(flame_index, _)| *flame_index);
    }

    // 5 Return the result.
    result
}
