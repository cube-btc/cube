use crate::inscriptive::flame_manager::errors::construction_error::FMConstructionError;
use crate::inscriptive::flame_manager::flame::flame::Flame;
use crate::inscriptive::flame_manager::flame_config::flame_config::FMAccountFlameConfig;
use crate::inscriptive::flame_manager::flame_set::flame_set::FMAccountFlameSet;
use crate::operative::Chain;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Account key.
type AccountKey = [u8; 32];

/// Rollup height.
type AtRollupHeight = u64;

/// Special db key for the account flame config (0x00..).
const ACCOUNT_FLAME_CONFIG_SPECIAL_DB_KEY: [u8; 1] = [0x00; 1];

/// Special db key for the account flame set (0x01..).
const ACCOUNT_FLAME_SET_SPECIAL_DB_KEY: [u8; 1] = [0x01; 1];

/// Flame manager.
#[allow(dead_code)]
pub struct FlameManager {
    // In-memory account flame configs.
    account_flame_configs: HashMap<AccountKey, FMAccountFlameConfig>,

    // In-memory account flame sets.
    account_flame_sets: HashMap<AccountKey, FMAccountFlameSet>,

    // In-memory projected flames.
    projected_flames: HashMap<AtRollupHeight, Vec<Flame>>,

    // On-disk accounts database.
    on_disk_accounts: sled::Db,

    // On-disk projected flames database.
    on_disk_projected_flames: sled::Db,
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

        // 2 Open the projected flames db.
        let projected_flames_db_path =
            format!("storage/{}/flames/projected_flames", chain.to_string());
        let projected_flames_db = sled::open(projected_flames_db_path)
            .map_err(FMConstructionError::ProjectedFlamesDBOpenError)?;

        // 3 Initialize the in-memory account flame configs and sets.
        let mut account_flame_configs = HashMap::<AccountKey, FMAccountFlameConfig>::new();
        let mut account_flame_sets = HashMap::<AccountKey, FMAccountFlameSet>::new();

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

            // 4.3 Initialize the account flame config and set.
            let mut account_flame_config: Option<FMAccountFlameConfig> = None;
            let mut account_flame_set: Option<FMAccountFlameSet> = None;

            // 4.4 Iterate over all items in the tree.
            for item in tree.iter() {
                // 4.4.1 Get the key and value.
                let (key, value) = match item {
                    Ok((k, v)) => (k, v),
                    Err(e) => {
                        return Err(FMConstructionError::AccountsTreeIterError(account_key, e));
                    }
                };

                // 4.4.2 Convert the tree key to the single db key byte.
                let key_byte: [u8; 1] = key.as_ref().try_into().map_err(|_| {
                    FMConstructionError::UnableToDeserializeAccountDbKeyByteFromTreeKey(
                        account_key,
                        key.to_vec(),
                    )
                })?;

                // 4.4.3 Match the db key byte.
                match key_byte {
                    // 0x00 key byte represents the account flame config.
                    ACCOUNT_FLAME_CONFIG_SPECIAL_DB_KEY => {
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
                    }
                    // 0x01 key byte represents the account flame set.
                    ACCOUNT_FLAME_SET_SPECIAL_DB_KEY => {
                        if value.as_ref().len() > 0 {
                            // Deserialize the flame set from bytes.
                            let flame_set_deserialized = FMAccountFlameSet::from_bytes(value.as_ref())
                                .ok_or(FMConstructionError::UnableToDeserializeAccountFlameSetBytesFromTreeValue(
                                    account_key,
                                    value.to_vec(),
                                ))?;

                            // Update the account flame set.
                            account_flame_set = Some(flame_set_deserialized);
                        }
                    }
                    // Invalid db key byte.
                    _ => {
                        return Err(FMConstructionError::InvalidAccountDbKeyByte(
                            account_key,
                            key.to_vec(),
                        ));
                    }
                }
            }

            // 4.5 Insert the account flame config if it exists.
            if let Some(config) = account_flame_config {
                account_flame_configs.insert(account_key, config);
            }

            // 4.6 Insert the account flame set if it exists.
            if let Some(set) = account_flame_set {
                account_flame_sets.insert(account_key, set);
            }
        }

        // 5 Initialize the in-memory projected flames.
        let mut projected_flames = HashMap::<AtRollupHeight, Vec<Flame>>::new();

        // 6 Collect projected flames from the projected flames database.
        for tree_name in projected_flames_db.tree_names() {
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
            let tree = projected_flames_db
                .open_tree(tree_name)
                .map_err(|e| FMConstructionError::ProjectedFlamesTreeOpenError(rollup_height, e))?;

            // 6.3 Initialize the list of flames for this rollup height.
            let mut flames = Vec::<Flame>::new();

            // 6.4 Iterate over all items in the tree.
            for item in tree.iter() {
                // 6.4.1 Get the key and value.
                let (_key, value) = match item {
                    Ok((k, v)) => (k, v),
                    Err(e) => {
                        return Err(FMConstructionError::ProjectedFlamesTreeIterError(
                            rollup_height,
                            e,
                        ));
                    }
                };

                // 6.4.2 Deserialize the flame from the value bytes.
                let flame = Flame::from_bytes(value.as_ref()).ok_or(
                    FMConstructionError::UnableToDeserializeProjectedFlameBytesFromTreeValue(
                        rollup_height,
                        value.to_vec(),
                    ),
                )?;

                // 6.4.3 Push the flame.
                flames.push(flame);
            }

            // 6.5 Insert the projected flames for this rollup height.
            projected_flames.insert(rollup_height, flames);
        }

        // 7 Construct the flame manager.
        let flame_manager = FlameManager {
            account_flame_configs,
            account_flame_sets,
            projected_flames,
            on_disk_accounts: accounts_db,
            on_disk_projected_flames: projected_flames_db,
        };

        // 8 Guard the flame manager.
        let guarded_flame_manager = Arc::new(Mutex::new(flame_manager));

        // 9 Return the guarded flame manager.
        Ok(guarded_flame_manager)
    }
}
