use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::flame_manager::algorithms::flame_selection::flame_selection::return_flames_to_fund;
use crate::inscriptive::flame_manager::algorithms::flame_sorting::flame_sorting::sort_flames;
use crate::inscriptive::flame_manager::delta::delta::FMDelta;
use crate::inscriptive::flame_manager::errors::apply_changes_error::FMApplyChangesError;
use crate::inscriptive::flame_manager::errors::construction_error::FMConstructionError;
use crate::inscriptive::flame_manager::errors::register_account_error::FMRegisterAccountError;
use crate::inscriptive::flame_manager::errors::update_account_flame_config_error::FMUpdateAccountFlameConfigError;
use crate::inscriptive::flame_manager::flame::flame::Flame;
use crate::inscriptive::flame_manager::flame_config::flame_config::FMAccountFlameConfig;
use crate::operative::Chain;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

/// Account key.
type AccountKey = [u8; 32];

/// Projector height.
type ProjectorHeight = u64;

/// Flame index.
pub type FlameIndex = u32;

/// Flame projection template.
pub type FlameProjectionTemplate = Vec<(AccountKey, Vec<(FlameIndex, Flame)>)>;

/// Special db key for the account flame config (0x00..).
const ACCOUNT_FLAME_CONFIG_SPECIAL_DB_KEY: [u8; 1] = [0x00; 1];

/// Flame manager.
#[allow(dead_code)]
pub struct FlameManager {
    // In-memory account flame configs.
    in_memory_account_flame_configs: HashMap<AccountKey, FMAccountFlameConfig>,

    // In-memory flame set.
    in_memory_flame_set: HashMap<AccountKey, HashMap<ProjectorHeight, Vec<(FlameIndex, Flame)>>>,

    // On-disk accounts database.
    on_disk_accounts: sled::Db,

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

        // 2 Initialize the in-memory account flame configs and flame set.
        let mut in_memory_account_flame_configs =
            HashMap::<AccountKey, FMAccountFlameConfig>::new();
        let mut in_memory_flame_set =
            HashMap::<AccountKey, HashMap<ProjectorHeight, Vec<(FlameIndex, Flame)>>>::new();

        // 3 Collect account flame configs and sets from the accounts database.
        for tree_name in accounts_db.tree_names() {
            // 3.1 Deserialize account key bytes from tree name.
            let account_key: [u8; 32] = match tree_name.as_ref().try_into() {
                Ok(key) => key,
                Err(_) => {
                    // Tree name is probably '__sled__default'. Skip it.
                    continue;
                }
            };

            // 3.2 Open the tree.
            let tree = accounts_db
                .open_tree(tree_name)
                .map_err(|e| FMConstructionError::AccountsTreeOpenError(account_key, e))?;

            // 3.3 Initialize the account flame config and flames grouped by rollup height.
            let mut account_flame_config: Option<FMAccountFlameConfig> = None;
            let mut account_flames_by_height: HashMap<ProjectorHeight, Vec<(FlameIndex, Flame)>> =
                HashMap::new();

            // 3.4 Iterate over all items in the tree.
            for item in tree.iter() {
                // 3.4.1 Get the key and value.
                let (key, value) = match item {
                    Ok((k, v)) => (k, v),
                    Err(e) => {
                        return Err(FMConstructionError::AccountsTreeIterError(account_key, e));
                    }
                };

                // 3.4.2 Check if this is the special config key or a flame index key.
                if key.as_ref().len() == 1 {
                    // 3.4.2.1 Convert the key to a byte.
                    let key_byte: [u8; 1] = key.as_ref().try_into().map_err(|_| {
                        FMConstructionError::UnableToDeserializeAccountDbKeyByteFromTreeKey(
                            account_key,
                            key.to_vec(),
                        )
                    })?;

                    // 3.4.2.2 Check if the key byte is the special config key (0x00).
                    if key_byte == ACCOUNT_FLAME_CONFIG_SPECIAL_DB_KEY {
                        // 3.4.2.2.1 Check if the value is not empty.
                        if value.as_ref().len() > 0 {
                            // 3.4.2.2.1.1 Deserialize the flame config from bytes.
                            let flame_config_deserialized = FMAccountFlameConfig::from_bytes(value.as_ref())
                                .ok_or(FMConstructionError::UnableToDeserializeAccountFlameConfigBytesFromTreeValue(
                                    account_key,
                                    value.to_vec(),
                                ))?;

                            // 3.4.2.2.1.2 Update the account flame config.
                            account_flame_config = Some(flame_config_deserialized);
                        }
                        continue;
                    }
                }

                // 3.4.3 Convert the tree key to 12 bytes: 8-byte height + 4-byte index.
                if key.as_ref().len() != 12 {
                    return Err(FMConstructionError::InvalidAccountDbKeyByte(
                        account_key,
                        key.to_vec(),
                    ));
                }

                // 3.4.3.1 Extract the rollup height bytes (first 8 bytes) and convert to u64.
                let rollup_height_bytes: [u8; 8] = key.as_ref()[0..8].try_into().unwrap();
                let rollup_height = u64::from_le_bytes(rollup_height_bytes);

                // 3.4.3.2 Extract the flame index bytes (last 4 bytes) and convert to u32.
                let flame_index_bytes: [u8; 4] = key.as_ref()[8..12].try_into().unwrap();
                let flame_index = u32::from_le_bytes(flame_index_bytes);

                // 3.4.4 Deserialize the value: literal flame bytes (no prefix).
                let flame = Flame::from_bytes(value.as_ref()).ok_or(
                    FMConstructionError::UnableToDeserializeAccountFlameSetBytesFromTreeValue(
                        account_key,
                        value.to_vec(),
                    ),
                )?;

                // 3.4.5 Store the flame grouped by rollup height.
                account_flames_by_height
                    .entry(rollup_height)
                    .or_insert_with(Vec::new)
                    .push((flame_index, flame));
            }

            // 3.5 Insert the account flame config if it exists.
            if let Some(config) = account_flame_config {
                in_memory_account_flame_configs.insert(account_key, config);
            }

            // 3.6 Sort flames by index within each rollup height and insert.
            if !account_flames_by_height.is_empty() {
                for flames in account_flames_by_height.values_mut() {
                    flames.sort_by_key(|(flame_index, _)| *flame_index);
                }
                in_memory_flame_set.insert(account_key, account_flames_by_height);
            }
        }

        // 4 Construct the flame manager.
        let flame_manager = FlameManager {
            in_memory_account_flame_configs,
            in_memory_flame_set,
            on_disk_accounts: accounts_db,
            delta: FMDelta::fresh_new(),
            backup_of_delta: FMDelta::fresh_new(),
        };

        // 5 Guard the flame manager.
        let guarded_flame_manager = Arc::new(Mutex::new(flame_manager));

        // 6 Return the guarded flame manager.
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

    /// Prepares the flame manager prior to each execution.
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
        // 1 Try to get from the delta first (ephemeral updates).
        if let Some(flame_config) = self.delta.updated_flame_configs.get(&account_key) {
            return Some(flame_config.clone());
        }

        // 2 Try to get from the delta's new accounts to register (ephemeral registrations).
        if let Some((_, flame_config)) = self
            .delta
            .new_accounts_to_register
            .iter()
            .find(|(key, _)| key == &account_key)
        {
            if let Some(flame_config) = flame_config {
                return Some(flame_config.clone());
            }
        }

        // 3 And then try to get from the permanent in-memory states.
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
        match self.in_memory_flame_set.get(&account_key) {
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

    /// Applies the changes to the flame manager.
    pub async fn apply_changes(
        &mut self,
        coin_manager: &COIN_MANAGER,
        new_projector_height: ProjectorHeight,
        projector_expiry_height: ProjectorHeight,
    ) -> Result<FlameProjectionTemplate, FMApplyChangesError> {
        // 1 Register new accounts.
        for (account_key, flame_config) in self.delta.new_accounts_to_register.iter() {
            // 1.1 On-disk insertion.
            {
                // 1.1.1 Open the tree for the account.
                let tree = self.on_disk_accounts.open_tree(account_key).map_err(|e| {
                    FMApplyChangesError::AccountTreeOpenError(account_key.to_owned(), e)
                })?;

                // 1.1.2 Insert the flame config on-disk if present.
                if let Some(flame_config) = flame_config {
                    // 1.1.2.1 Serialize the flame config to bytes.
                    let flame_config_bytes = flame_config.to_bytes();

                    // 1.1.2.2 Insert the flame config on-disk.
                    tree.insert(ACCOUNT_FLAME_CONFIG_SPECIAL_DB_KEY, flame_config_bytes)
                        .map_err(|e| {
                            FMApplyChangesError::AccountFlameConfigInsertError(
                                account_key.to_owned(),
                                e,
                            )
                        })?;
                }
            }

            // 1.2 In-memory insertion.
            {
                // 1.2.1 Insert the flame config into the in-memory account flame configs if present.
                if let Some(flame_config) = flame_config {
                    self.in_memory_account_flame_configs
                        .insert(account_key.to_owned(), flame_config.clone());
                }

                // 1.2.2 Initialize an empty flame set for the account in-memory.
                self.in_memory_flame_set
                    .insert(account_key.to_owned(), HashMap::new());
            }
        }

        // 2 Update account flame configs.
        for (account_key, flame_config) in self.delta.updated_flame_configs.iter() {
            // 2.1 Check if the account exists in memory.
            if !self
                .in_memory_account_flame_configs
                .contains_key(account_key)
            {
                return Err(FMApplyChangesError::AccountNotFoundInMemory(
                    account_key.to_owned(),
                ));
            }

            // 2.2 On-disk update.
            {
                // 2.2.1 Open the tree for the account.
                let tree = self.on_disk_accounts.open_tree(account_key).map_err(|e| {
                    FMApplyChangesError::AccountTreeOpenError(account_key.to_owned(), e)
                })?;

                // 2.2.2 Serialize the flame config to bytes.
                let flame_config_bytes = flame_config.to_bytes();

                // 2.2.3 Update the flame config on-disk.
                tree.insert(ACCOUNT_FLAME_CONFIG_SPECIAL_DB_KEY, flame_config_bytes)
                    .map_err(|e| {
                        FMApplyChangesError::AccountFlameConfigInsertError(
                            account_key.to_owned(),
                            e,
                        )
                    })?;
            }

            // 2.3 In-memory update.
            {
                // 2.3.1 Update the flame config.
                self.in_memory_account_flame_configs
                    .insert(account_key.to_owned(), flame_config.clone());
            }
        }

        // 3 Collect the coingap accounts set.
        let coingap_accounts_list: HashSet<AccountKey> = {
            // 3.1 Initialize the coingap accounts set.
            let mut coingap_accounts_list: HashSet<AccountKey> = HashSet::new();

            // 3.2 Get affected coin manager accounts.
            let affected_coin_manager_accounts: Vec<AccountKey> = {
                // 3.2.1 Lock the coin manager.
                let _coin_manager = coin_manager.lock().await;

                // 3.2.2 Get the affected coin manager accounts list.
                _coin_manager.get_coingap_accounts_list()
            };

            // 3.3 Extend the overall affected accounts set with the affected coin manager accounts.
            coingap_accounts_list.extend(affected_coin_manager_accounts);

            // 3.4 Get affected expired flames accounts.
            let affected_expired_flames_accounts: HashSet<AccountKey> = {
                // 3.4.1 Initialize the affected expired flames accounts set.
                let mut affected_expired_flames_accounts: HashSet<AccountKey> = HashSet::new();

                // 3.4.2 Iterate over all in-memory flame sets.
                for (account_key, account_flame_set) in self.in_memory_flame_set.iter() {
                    // 3.4.2.1 Iterate over all rollup heights in the flame set.
                    'inner_loop: for (projector_height, _) in account_flame_set.iter() {
                        // 3.4.2.1.1 Check if the rollup height is below or equal to the projector rollup height.
                        if *projector_height <= projector_expiry_height {
                            // 3.4.2.1.1.1 Insert the account key into the affected expired flames accounts set.
                            affected_expired_flames_accounts.insert(account_key.to_owned());

                            // 3.4.2.1.1.2 Break the inner loop.
                            break 'inner_loop;
                        }
                    }
                }

                // 3.4.3 Return the affected expired flames accounts set.
                affected_expired_flames_accounts
            };

            // 3.5 Extend the coingap accounts set with the affected expired flames accounts.
            coingap_accounts_list.extend(affected_expired_flames_accounts);

            // 3.6 Return the coingap accounts set.
            coingap_accounts_list
        };

        // 4 Initialize the new flames to insert.
        let mut new_flames_to_insert: HashMap<AccountKey, Vec<Flame>> = HashMap::new();

        // 5 Collect new flames to insert and prune expired flames in the meantime.
        {
            // 5.1 Lock the coin manager.
            let _coin_manager = coin_manager.lock().await;

            // 5.2 Iterate over all affected accounts.
            'coingap_accounts_loop: for account_key in coingap_accounts_list {
                // 5.2.1 Get the account flame config.
                let account_flame_config: FMAccountFlameConfig =
                    match self.get_account_flame_config(account_key) {
                        // 5.2.1.a The account flame config is set.
                        Some(flame_config) => flame_config,

                        // 5.2.1.b The account flame config is not set.
                        None => {
                            // 5.2.1.b.1 Continue to the next affected account.
                            continue 'coingap_accounts_loop;
                        }
                    };

                // 5.2.2 Open the tree for the account.
                let tree = self
                    .on_disk_accounts
                    .open_tree(account_key)
                    .map_err(|e| FMApplyChangesError::AccountTreeOpenError(account_key, e))?;

                // 5.2.3 Get the in-memory mutable flame set.
                let account_flame_set_mut = self
                    .in_memory_flame_set
                    .get_mut(&account_key)
                    .expect("This should never happen.");

                // 5.2.4 Initialize the list of pruned flame db keys.
                let mut pruned_flame_db_keys: Vec<[u8; 12]> = Vec::new();

                // 5.2.5 Iterate over all rollup heights in the flame set.
                for (projector_height, flames) in account_flame_set_mut.clone().iter() {
                    // 5.2.5.1 Check if the rollup height is below or equal to the projector rollup height.
                    if *projector_height <= projector_expiry_height {
                        // 5.2.5.1.1 Iterate over all flames in the rollup height.
                        for (flame_index, _) in flames.iter() {
                            // 5.2.5.1.1.1 Convert the rollup height and flame index to a 12 byte db key.
                            let flame_db_key: [u8; 12] = {
                                // 5.2.5.1.1.1.1 Initialize the flame db key.
                                let mut flame_db_key: [u8; 12] = [0; 12];

                                // 5.2.5.1.1.1.2 Copy the rollup height bytes to the flame db key.
                                flame_db_key[0..8].copy_from_slice(&projector_height.to_le_bytes());

                                // 5.2.5.1.1.1.3 Copy the flame index bytes to the flame db key.
                                flame_db_key[8..12].copy_from_slice(&flame_index.to_le_bytes());

                                // 5.2.5.1.1.1.4 Return the flame db key.
                                flame_db_key
                            };

                            // 5.2.5.1.1.2 Insert the flame db key into the list of pruned flame db keys.
                            pruned_flame_db_keys.push(flame_db_key);
                        }

                        // 5.2.5.1.2 Prune expired flames from the in-memory flame set.
                        account_flame_set_mut.remove(projector_height);
                    }
                }

                // 5.2.6 Prune expired flames from the on-disk flame set.
                for flame_db_key_to_prune in pruned_flame_db_keys {
                    // 5.2.6.1 Remove the flame from the tree.
                    tree.remove(flame_db_key_to_prune).map_err(|e| {
                        FMApplyChangesError::AccountRemoveFlameFromDiskTreeError(
                            account_key,
                            flame_db_key_to_prune,
                            e,
                        )
                    })?;
                }

                // 5.2.7 Get the target flame value for the account.
                let account_target_flame_value_in_satoshis: u64 = {
                    _coin_manager
                        .get_account_target_flame_value_in_satoshis(account_key)
                        .ok_or(
                            FMApplyChangesError::AccountTargetFlameValueCouldNotBeRetrieved(
                                account_key,
                            ),
                        )?
                };

                // 5.2.8 Get the current flame set sum value.
                let account_current_flame_set_sum_value_in_satoshis: u64 = {
                    // 5.2.8.1 Initialize the current flame set sum value.
                    let mut account_current_flame_set_sum_value_in_satoshis: u64 = 0;

                    // 5.2.8.2 Iterate over all flames in the flame set.
                    for (_, flames) in account_flame_set_mut.iter() {
                        // 5.2.8.2.1 Add the flame value to the current flame set sum value.
                        for (_, flame) in flames.iter() {
                            account_current_flame_set_sum_value_in_satoshis +=
                                flame.satoshi_amount();
                        }
                    }

                    // 5.2.8.3 Return the current flame set sum value.
                    account_current_flame_set_sum_value_in_satoshis
                };

                // 5.2.9 Retrieve the flames to fund.
                let flames_to_fund: Vec<Flame> = return_flames_to_fund(
                    &account_flame_config,
                    account_target_flame_value_in_satoshis,
                    account_current_flame_set_sum_value_in_satoshis,
                )
                .unwrap_or_default();

                // 5.2.10 Insert the flames to fund into the in-memory flame set.
                new_flames_to_insert.insert(account_key, flames_to_fund);
            }
        }

        // 6 Sort the new flames to insert.
        let sorted_new_flames_to_insert: Vec<(AccountKey, Vec<(FlameIndex, Flame)>)> =
            sort_flames(new_flames_to_insert);

        // 7 Apply changes to the flame set (inserts-only, pruning is already done).
        for (account_key, flames) in sorted_new_flames_to_insert.iter() {
            // 7.1 Insert in-memory.
            {
                // 7.1.1 Get the mutable flame set.
                let flame_set_mut = self
                    .in_memory_flame_set
                    .get_mut(account_key)
                    .expect("This should never happen.");

                // 7.1.2 Iterate and insert the new flames into the flame set.
                flame_set_mut.insert(new_projector_height, flames.to_owned());
            }

            // 7.2 Insert on-disk.
            {
                // 7.2.1 Open the tree for the account.
                let tree = self.on_disk_accounts.open_tree(account_key).map_err(|e| {
                    FMApplyChangesError::AccountTreeOpenError(account_key.to_owned(), e)
                })?;

                // 7.2.2 Iterate and insert the new flames into the on-disk flame set.
                for (flame_index, flame) in flames.iter() {
                    // 7.2.2.1 Convert the rollup height and flame index to a 12 byte db key.
                    let flame_db_key: [u8; 12] = {
                        // 7.2.2.1.1 Initialize the flame db key.
                        let mut flame_db_key: [u8; 12] = [0; 12];

                        // 7.2.2.1.2 Copy the rollup height bytes to the flame db key.
                        flame_db_key[0..8].copy_from_slice(&new_projector_height.to_le_bytes());

                        // 7.2.2.1.3 Copy the flame index bytes to the flame db key.
                        flame_db_key[8..12].copy_from_slice(&flame_index.to_le_bytes());

                        // 7.2.2.1.4 Return the flame db key.
                        flame_db_key
                    };

                    // 7.2.2.2 Insert the new flame into the on-disk flame set.
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

        // 8 Return the result.
        Ok(sorted_new_flames_to_insert)
    }

    /// Clears all epheremal changes from the delta.
    pub fn flush_delta(&mut self) {
        // Clear the epheremal changes from the delta.
        self.delta.flush();

        // Clear the epheremal changes from the backup.
        self.backup_of_delta.flush();
    }
}
