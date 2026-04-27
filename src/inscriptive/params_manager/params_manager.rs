use crate::inscriptive::params_manager::params_holder::params_holder::ParamsHolder;
use crate::operative::run_args::chain::Chain;
use std::sync::{Arc, Mutex};

const ACCOUNT_CAN_INITIALLY_DEPLOY_LIQUIDITY_SPECIAL_DB_KEY: [u8; 1] = [0x00; 1];
const ACCOUNT_CAN_INITIALLY_DEPLOY_CONTRACT_SPECIAL_DB_KEY: [u8; 1] = [0x01; 1];
const MOVE_ENTRY_BASE_FEE_SPECIAL_DB_KEY: [u8; 1] = [0x02; 1];
const CALL_ENTRY_BASE_FEE_SPECIAL_DB_KEY: [u8; 1] = [0x03; 1];
const CALL_ENTRY_PPM_CALLDATA_BYTESIZE_FEE_SPECIAL_DB_KEY: [u8; 1] = [0x04; 1];
const LIFTUP_ENTRY_BASE_FEE_SPECIAL_DB_KEY: [u8; 1] = [0x05; 1];
const LIFTUP_ENTRY_PER_LIFT_BASE_FEE_SPECIAL_DB_KEY: [u8; 1] = [0x06; 1];
const MOVE_PPM_LIQUIDITY_FEE_SPECIAL_DB_KEY: [u8; 1] = [0x07; 1];
const IN_CALL_PPM_LIQUIDITY_FEE_SPECIAL_DB_KEY: [u8; 1] = [0x08; 1];

const PARAMS_HOLDER_TREE_NAME: [u8; 13] = *b"params_holder";

#[derive(Clone)]
struct ParamsManagerDelta {
    updated_params_holder: Option<ParamsHolder>,
}

impl ParamsManagerDelta {
    fn fresh_new() -> Self {
        Self {
            updated_params_holder: None,
        }
    }

    fn flush(&mut self) {
        self.updated_params_holder = None;
    }
}

/// A manager for protocol-level params.
pub struct ParamsManager {
    in_memory_params_holder: ParamsHolder,
    on_disk_params: sled::Db,
    delta: ParamsManagerDelta,
    backup_of_delta: ParamsManagerDelta,
}

/// Guarded 'ParamsManager'.
#[allow(non_camel_case_types)]
pub type PARAMS_MANAGER = Arc<Mutex<ParamsManager>>;

impl ParamsManager {
    /// Creates a new params manager.
    pub fn new(chain: Chain) -> Result<PARAMS_MANAGER, sled::Error> {
        // 1 Open params db.
        let params_db_path = format!("storage/{}/params", chain.to_string());
        let params_db = sled::open(params_db_path)?;

        // 2 Start with the default params holder.
        let mut params_holder = ParamsHolder::fresh_new();

        // 3 Open the params holder tree.
        let tree = params_db.open_tree(PARAMS_HOLDER_TREE_NAME)?;

        // 4 Collect persisted values.
        for item in tree.iter() {
            let (key, value) = item?;

            let key_byte: [u8; 1] = match key.as_ref().try_into() {
                Ok(byte) => byte,
                Err(_) => continue,
            };

            match key_byte {
                ACCOUNT_CAN_INITIALLY_DEPLOY_LIQUIDITY_SPECIAL_DB_KEY => {
                    params_holder.account_can_initially_deploy_liquidity =
                        value.as_ref().first().copied().unwrap_or(1) != 0;
                }
                ACCOUNT_CAN_INITIALLY_DEPLOY_CONTRACT_SPECIAL_DB_KEY => {
                    params_holder.account_can_initially_deploy_contract =
                        value.as_ref().first().copied().unwrap_or(1) != 0;
                }
                MOVE_ENTRY_BASE_FEE_SPECIAL_DB_KEY => {
                    if let Ok(bytes) = value.as_ref().try_into() {
                        params_holder.move_entry_base_fee = u64::from_le_bytes(bytes);
                    }
                }
                CALL_ENTRY_BASE_FEE_SPECIAL_DB_KEY => {
                    if let Ok(bytes) = value.as_ref().try_into() {
                        params_holder.call_entry_base_fee = u64::from_le_bytes(bytes);
                    }
                }
                CALL_ENTRY_PPM_CALLDATA_BYTESIZE_FEE_SPECIAL_DB_KEY => {
                    if let Ok(bytes) = value.as_ref().try_into() {
                        params_holder.call_entry_ppm_calldata_bytesize_fee =
                            u64::from_le_bytes(bytes);
                    }
                }
                LIFTUP_ENTRY_BASE_FEE_SPECIAL_DB_KEY => {
                    if let Ok(bytes) = value.as_ref().try_into() {
                        params_holder.liftup_entry_base_fee = u64::from_le_bytes(bytes);
                    }
                }
                LIFTUP_ENTRY_PER_LIFT_BASE_FEE_SPECIAL_DB_KEY => {
                    if let Ok(bytes) = value.as_ref().try_into() {
                        params_holder.liftup_entry_per_lift_base_fee = u64::from_le_bytes(bytes);
                    }
                }
                MOVE_PPM_LIQUIDITY_FEE_SPECIAL_DB_KEY => {
                    if let Ok(bytes) = value.as_ref().try_into() {
                        params_holder.move_ppm_liquidity_fee = u64::from_le_bytes(bytes);
                    }
                }
                IN_CALL_PPM_LIQUIDITY_FEE_SPECIAL_DB_KEY => {
                    if let Ok(bytes) = value.as_ref().try_into() {
                        params_holder.in_call_ppm_liquidity_fee = u64::from_le_bytes(bytes);
                    }
                }
                _ => (),
            }
        }

        // 5 Build and guard the manager.
        let manager = ParamsManager {
            in_memory_params_holder: params_holder,
            on_disk_params: params_db,
            delta: ParamsManagerDelta::fresh_new(),
            backup_of_delta: ParamsManagerDelta::fresh_new(),
        };

        Ok(Arc::new(Mutex::new(manager)))
    }

    /// Clones the delta into the backup.
    fn backup_delta(&mut self) {
        self.backup_of_delta = self.delta.clone();
    }

    /// Restores the delta from the backup.
    fn restore_delta(&mut self) {
        self.delta = self.backup_of_delta.clone();
    }

    /// Prepares params manager prior to each execution.
    pub fn pre_execution(&mut self) {
        self.backup_delta();
    }

    /// Returns the current params holder.
    pub fn get_params_holder(&self) -> ParamsHolder {
        if let Some(holder) = self.delta.updated_params_holder.as_ref() {
            return holder.clone();
        }

        self.in_memory_params_holder.clone()
    }

    fn get_mut_ephemeral_params_holder(&mut self) -> &mut ParamsHolder {
        if self.delta.updated_params_holder.is_none() {
            self.delta.updated_params_holder = Some(self.in_memory_params_holder.clone());
        }

        self.delta.updated_params_holder.as_mut().unwrap()
    }

    pub fn set_account_can_initially_deploy_liquidity(&mut self, value: bool) {
        self.get_mut_ephemeral_params_holder()
            .account_can_initially_deploy_liquidity = value;
    }

    pub fn set_account_can_initially_deploy_contract(&mut self, value: bool) {
        self.get_mut_ephemeral_params_holder()
            .account_can_initially_deploy_contract = value;
    }

    pub fn set_move_entry_base_fee(&mut self, value: u64) {
        self.get_mut_ephemeral_params_holder().move_entry_base_fee = value;
    }

    pub fn set_call_entry_base_fee(&mut self, value: u64) {
        self.get_mut_ephemeral_params_holder().call_entry_base_fee = value;
    }

    pub fn set_call_entry_ppm_calldata_bytesize_fee(&mut self, value: u64) {
        self.get_mut_ephemeral_params_holder()
            .call_entry_ppm_calldata_bytesize_fee = value;
    }

    pub fn set_liftup_entry_base_fee(&mut self, value: u64) {
        self.get_mut_ephemeral_params_holder().liftup_entry_base_fee = value;
    }

    pub fn set_liftup_entry_per_lift_base_fee(&mut self, value: u64) {
        self.get_mut_ephemeral_params_holder()
            .liftup_entry_per_lift_base_fee = value;
    }

    pub fn set_move_ppm_liquidity_fee(&mut self, value: u64) {
        self.get_mut_ephemeral_params_holder().move_ppm_liquidity_fee = value;
    }

    pub fn set_in_call_ppm_liquidity_fee(&mut self, value: u64) {
        self.get_mut_ephemeral_params_holder().in_call_ppm_liquidity_fee = value;
    }

    /// Reverts the epheremal changes associated with the last execution.
    pub fn rollback_last(&mut self) {
        self.restore_delta();
    }

    /// Applies all epheremal changes from delta into permanent in-memory and on-disk state.
    pub fn apply_changes(&mut self) -> Result<(), sled::Error> {
        if let Some(ephemeral_params_holder) = self.delta.updated_params_holder.as_ref() {
            let tree = self.on_disk_params.open_tree(PARAMS_HOLDER_TREE_NAME)?;

            tree.insert(
                ACCOUNT_CAN_INITIALLY_DEPLOY_LIQUIDITY_SPECIAL_DB_KEY,
                [if ephemeral_params_holder.account_can_initially_deploy_liquidity {
                    1u8
                } else {
                    0u8
                }]
                .as_slice(),
            )?;
            tree.insert(
                ACCOUNT_CAN_INITIALLY_DEPLOY_CONTRACT_SPECIAL_DB_KEY,
                [if ephemeral_params_holder.account_can_initially_deploy_contract {
                    1u8
                } else {
                    0u8
                }]
                .as_slice(),
            )?;
            tree.insert(
                MOVE_ENTRY_BASE_FEE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .move_entry_base_fee
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                CALL_ENTRY_BASE_FEE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .call_entry_base_fee
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                CALL_ENTRY_PPM_CALLDATA_BYTESIZE_FEE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .call_entry_ppm_calldata_bytesize_fee
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                LIFTUP_ENTRY_BASE_FEE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .liftup_entry_base_fee
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                LIFTUP_ENTRY_PER_LIFT_BASE_FEE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .liftup_entry_per_lift_base_fee
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                MOVE_PPM_LIQUIDITY_FEE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .move_ppm_liquidity_fee
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                IN_CALL_PPM_LIQUIDITY_FEE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .in_call_ppm_liquidity_fee
                    .to_le_bytes()
                    .to_vec(),
            )?;

            self.in_memory_params_holder = ephemeral_params_holder.clone();
        }

        Ok(())
    }

    /// Clears all epheremal changes from delta and backup.
    pub fn flush_delta(&mut self) {
        self.delta.flush();
        self.backup_of_delta.flush();
    }
}

/// Erases the params manager by db path.
pub fn erase_params_manager(chain: Chain) {
    let params_db_path = format!("storage/{}/params", chain.to_string());
    let _ = std::fs::remove_dir_all(params_db_path);
}
