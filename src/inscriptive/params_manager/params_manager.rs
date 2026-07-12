use crate::inscriptive::params_manager::params_holder::params_holder::ParamsHolder;
use crate::inscriptive::params_manager::params_keys::*;
use crate::operative::run_args::chain::Chain;
use std::sync::{Arc, Mutex};

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
        let params_db_path = format!("storage/{}/params", chain);
        let params_db = sled::open(params_db_path)?;

        // 2 Start with the default params holder.
        let mut params_holder = ParamsHolder::origin_params_holder();

        // 3 Open the params holder tree.
        let tree = params_db.open_tree(PARAMS_HOLDER_TREE_NAME)?;

        // 4 Collect persisted values.
        for item in tree.iter() {
            let (key, value) = item?;

            let key_bytes: [u8; 2] = match key.as_ref().try_into() {
                Ok(bytes) => bytes,
                Err(_) => continue,
            };

            match key_bytes[0] {
                PARAMS_CATEGORY_CASUAL => match key_bytes[1] {
                    0x00 => {
                        params_holder.account_can_initially_deploy_liquidity =
                            value.as_ref().first().copied().unwrap_or(1) != 0;
                    }
                    0x01 => {
                        params_holder.account_can_initially_deploy_contract =
                            value.as_ref().first().copied().unwrap_or(1) != 0;
                    }
                    0x02 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.move_entry_base_fee = u64::from_le_bytes(bytes);
                        }
                    }
                    0x03 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.call_entry_base_fee = u64::from_le_bytes(bytes);
                        }
                    }
                    0x04 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.call_entry_ppm_calldata_bytesize_fee =
                                u64::from_le_bytes(bytes);
                        }
                    }
                    0x05 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.liftup_entry_base_fee = u64::from_le_bytes(bytes);
                        }
                    }
                    0x06 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.liftup_entry_per_lift_base_fee =
                                u64::from_le_bytes(bytes);
                        }
                    }
                    0x07 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.move_ppm_liquidity_fee = u64::from_le_bytes(bytes);
                        }
                    }
                    0x08 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.in_call_ppm_liquidity_fee = u64::from_le_bytes(bytes);
                        }
                    }
                    0x09 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.swapout_entry_base_fee = u64::from_le_bytes(bytes);
                        }
                    }
                    0x0A => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.config_entry_base_fee = u64::from_le_bytes(bytes);
                        }
                    }
                    0x0B => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.config_entry_per_config_byte_fee =
                                u64::from_le_bytes(bytes);
                        }
                    }
                    0x0C => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.deploy_entry_base_fee = u64::from_le_bytes(bytes);
                        }
                    }
                    0x0D => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.deploy_entry_per_program_byte_fee =
                                u64::from_le_bytes(bytes);
                        }
                    }
                    _ => (),
                },
                PARAMS_CATEGORY_OPCODE_OPS => match key_bytes[1] {
                    0x00 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_false = u64::from_le_bytes(bytes);
                        }
                    }
                    0x01 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_true = u64::from_le_bytes(bytes);
                        }
                    }
                    0x02 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_2 = u64::from_le_bytes(bytes);
                        }
                    }
                    0x03 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_3 = u64::from_le_bytes(bytes);
                        }
                    }
                    0x04 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_4 = u64::from_le_bytes(bytes);
                        }
                    }
                    0x05 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_5 = u64::from_le_bytes(bytes);
                        }
                    }
                    0x06 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_6 = u64::from_le_bytes(bytes);
                        }
                    }
                    0x07 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_7 = u64::from_le_bytes(bytes);
                        }
                    }
                    0x08 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_8 = u64::from_le_bytes(bytes);
                        }
                    }
                    0x09 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_9 = u64::from_le_bytes(bytes);
                        }
                    }
                    0x0A => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_10 = u64::from_le_bytes(bytes);
                        }
                    }
                    0x0B => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_11 = u64::from_le_bytes(bytes);
                        }
                    }
                    0x0C => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_12 = u64::from_le_bytes(bytes);
                        }
                    }
                    0x0D => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_13 = u64::from_le_bytes(bytes);
                        }
                    }
                    0x0E => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_14 = u64::from_le_bytes(bytes);
                        }
                    }
                    0x0F => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_15 = u64::from_le_bytes(bytes);
                        }
                    }
                    0x10 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_16 = u64::from_le_bytes(bytes);
                        }
                    }
                    0x11 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_pushdata_base = u64::from_le_bytes(bytes);
                        }
                    }
                    0x12 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_pushdata_per_byte =
                                u64::from_le_bytes(bytes);
                        }
                    }
                    0x13 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_nop = u64::from_le_bytes(bytes);
                        }
                    }
                    0x14 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_jump = u64::from_le_bytes(bytes);
                        }
                    }
                    0x15 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_if = u64::from_le_bytes(bytes);
                        }
                    }
                    0x16 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_notif = u64::from_le_bytes(bytes);
                        }
                    }
                    0x17 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_else = u64::from_le_bytes(bytes);
                        }
                    }
                    0x18 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_endif = u64::from_le_bytes(bytes);
                        }
                    }
                    0x19 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_verify = u64::from_le_bytes(bytes);
                        }
                    }
                    0x1A => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_returnall = u64::from_le_bytes(bytes);
                        }
                    }
                    0x1B => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_returnsome = u64::from_le_bytes(bytes);
                        }
                    }
                    0x1C => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_fail = u64::from_le_bytes(bytes);
                        }
                    }
                    0x1D => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_returnerr = u64::from_le_bytes(bytes);
                        }
                    }
                    0x1E => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_toaltstack = u64::from_le_bytes(bytes);
                        }
                    }
                    0x1F => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_fromaltstack = u64::from_le_bytes(bytes);
                        }
                    }
                    0x20 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_2drop = u64::from_le_bytes(bytes);
                        }
                    }
                    0x21 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_2dup = u64::from_le_bytes(bytes);
                        }
                    }
                    0x22 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_3dup = u64::from_le_bytes(bytes);
                        }
                    }
                    0x23 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_2over = u64::from_le_bytes(bytes);
                        }
                    }
                    0x24 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_2rot = u64::from_le_bytes(bytes);
                        }
                    }
                    0x25 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_2swap = u64::from_le_bytes(bytes);
                        }
                    }
                    0x26 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_ifdup = u64::from_le_bytes(bytes);
                        }
                    }
                    0x27 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_depth = u64::from_le_bytes(bytes);
                        }
                    }
                    0x28 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_drop = u64::from_le_bytes(bytes);
                        }
                    }
                    0x29 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_dup = u64::from_le_bytes(bytes);
                        }
                    }
                    0x2A => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_nip = u64::from_le_bytes(bytes);
                        }
                    }
                    0x2B => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_over = u64::from_le_bytes(bytes);
                        }
                    }
                    0x2C => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_pick = u64::from_le_bytes(bytes);
                        }
                    }
                    0x2D => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_roll = u64::from_le_bytes(bytes);
                        }
                    }
                    0x2E => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_rot = u64::from_le_bytes(bytes);
                        }
                    }
                    0x2F => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_swap = u64::from_le_bytes(bytes);
                        }
                    }
                    0x30 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_tuck = u64::from_le_bytes(bytes);
                        }
                    }
                    0x31 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_cat = u64::from_le_bytes(bytes);
                        }
                    }
                    0x32 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_split = u64::from_le_bytes(bytes);
                        }
                    }
                    0x33 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_left = u64::from_le_bytes(bytes);
                        }
                    }
                    0x34 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_right = u64::from_le_bytes(bytes);
                        }
                    }
                    0x35 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_size = u64::from_le_bytes(bytes);
                        }
                    }
                    0x36 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_invert = u64::from_le_bytes(bytes);
                        }
                    }
                    0x37 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_and = u64::from_le_bytes(bytes);
                        }
                    }
                    0x38 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_or = u64::from_le_bytes(bytes);
                        }
                    }
                    0x39 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_xor = u64::from_le_bytes(bytes);
                        }
                    }
                    0x3A => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_equal = u64::from_le_bytes(bytes);
                        }
                    }
                    0x3B => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_equalverify = u64::from_le_bytes(bytes);
                        }
                    }
                    0x3C => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_reverse = u64::from_le_bytes(bytes);
                        }
                    }
                    0x3D => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_1add = u64::from_le_bytes(bytes);
                        }
                    }
                    0x3E => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_1sub = u64::from_le_bytes(bytes);
                        }
                    }
                    0x3F => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_2mul = u64::from_le_bytes(bytes);
                        }
                    }
                    0x40 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_2div = u64::from_le_bytes(bytes);
                        }
                    }
                    0x41 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_addmod = u64::from_le_bytes(bytes);
                        }
                    }
                    0x42 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_mulmod = u64::from_le_bytes(bytes);
                        }
                    }
                    0x43 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_not = u64::from_le_bytes(bytes);
                        }
                    }
                    0x44 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_0notequal = u64::from_le_bytes(bytes);
                        }
                    }
                    0x45 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_add = u64::from_le_bytes(bytes);
                        }
                    }
                    0x46 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_sub = u64::from_le_bytes(bytes);
                        }
                    }
                    0x47 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_mul = u64::from_le_bytes(bytes);
                        }
                    }
                    0x48 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_div = u64::from_le_bytes(bytes);
                        }
                    }
                    0x49 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_lshift = u64::from_le_bytes(bytes);
                        }
                    }
                    0x4A => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_rshift = u64::from_le_bytes(bytes);
                        }
                    }
                    0x4B => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_booland = u64::from_le_bytes(bytes);
                        }
                    }
                    0x4C => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_boolor = u64::from_le_bytes(bytes);
                        }
                    }
                    0x4D => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_numequal = u64::from_le_bytes(bytes);
                        }
                    }
                    0x4E => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_numequalverify = u64::from_le_bytes(bytes);
                        }
                    }
                    0x4F => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_numnotequal = u64::from_le_bytes(bytes);
                        }
                    }
                    0x50 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_lessthan = u64::from_le_bytes(bytes);
                        }
                    }
                    0x51 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_greaterthan = u64::from_le_bytes(bytes);
                        }
                    }
                    0x52 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_lessthanorequal = u64::from_le_bytes(bytes);
                        }
                    }
                    0x53 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_greaterthanorequal =
                                u64::from_le_bytes(bytes);
                        }
                    }
                    0x54 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_min = u64::from_le_bytes(bytes);
                        }
                    }
                    0x55 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_max = u64::from_le_bytes(bytes);
                        }
                    }
                    0x56 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_within = u64::from_le_bytes(bytes);
                        }
                    }
                    0x57 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_mread = u64::from_le_bytes(bytes);
                        }
                    }
                    0x58 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_mwrite = u64::from_le_bytes(bytes);
                        }
                    }
                    0x59 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_mfree = u64::from_le_bytes(bytes);
                        }
                    }
                    0x5A => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_ripemd160 = u64::from_le_bytes(bytes);
                        }
                    }
                    0x5B => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_sha1 = u64::from_le_bytes(bytes);
                        }
                    }
                    0x5C => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_sha256 = u64::from_le_bytes(bytes);
                        }
                    }
                    0x5D => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_hash160 = u64::from_le_bytes(bytes);
                        }
                    }
                    0x5E => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_hash256 = u64::from_le_bytes(bytes);
                        }
                    }
                    0x5F => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_taggedhash_base = u64::from_le_bytes(bytes);
                        }
                    }
                    0x60 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_taggedhash_per_byte_gap =
                                u64::from_le_bytes(bytes);
                        }
                    }
                    0x61 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_taggedhash_output_len =
                                u64::from_le_bytes(bytes);
                        }
                    }
                    0x62 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_blake2bvar_base = u64::from_le_bytes(bytes);
                        }
                    }
                    0x63 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_blake2bvar_per_byte =
                                u64::from_le_bytes(bytes);
                        }
                    }
                    0x64 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_blake2svar_base = u64::from_le_bytes(bytes);
                        }
                    }
                    0x65 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_blake2svar_per_byte =
                                u64::from_le_bytes(bytes);
                        }
                    }
                    0x66 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_secpscalaradd = u64::from_le_bytes(bytes);
                        }
                    }
                    0x67 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_secpscalarmul = u64::from_le_bytes(bytes);
                        }
                    }
                    0x68 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_secppointadd = u64::from_le_bytes(bytes);
                        }
                    }
                    0x69 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_secppointmul = u64::from_le_bytes(bytes);
                        }
                    }
                    0x6A => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_pushsecpgeneratorpoint =
                                u64::from_le_bytes(bytes);
                        }
                    }
                    0x6B => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_iszerosecpscalar =
                                u64::from_le_bytes(bytes);
                        }
                    }
                    0x6C => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_isinfinitesecppoint =
                                u64::from_le_bytes(bytes);
                        }
                    }
                    0x6D => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_checkschnorrsig = u64::from_le_bytes(bytes);
                        }
                    }
                    0x6E => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_checkschnorrsigbip340 =
                                u64::from_le_bytes(bytes);
                        }
                    }
                    0x6F => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_checkblssig = u64::from_le_bytes(bytes);
                        }
                    }
                    0x70 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_checkblssigagg_base =
                                u64::from_le_bytes(bytes);
                        }
                    }
                    0x71 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_checkblssigagg_per_count =
                                u64::from_le_bytes(bytes);
                        }
                    }
                    0x72 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_caller = u64::from_le_bytes(bytes);
                        }
                    }
                    0x73 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_opsbudget = u64::from_le_bytes(bytes);
                        }
                    }
                    0x74 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_opscounter = u64::from_le_bytes(bytes);
                        }
                    }
                    0x75 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_opsprice = u64::from_le_bytes(bytes);
                        }
                    }
                    0x76 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_timestamp = u64::from_le_bytes(bytes);
                        }
                    }
                    0x77 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_call = u64::from_le_bytes(bytes);
                        }
                    }
                    0x78 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_callext = u64::from_le_bytes(bytes);
                        }
                    }
                    0x79 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_sread = u64::from_le_bytes(bytes);
                        }
                    }
                    0x7A => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_sfree = u64::from_le_bytes(bytes);
                        }
                    }
                    0x7B => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_swrite_base = u64::from_le_bytes(bytes);
                        }
                    }
                    0x7C => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_swrite_per_byte = u64::from_le_bytes(bytes);
                        }
                    }
                    0x7D => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_shadow_alloc = u64::from_le_bytes(bytes);
                        }
                    }
                    0x7E => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_shadow_dealloc = u64::from_le_bytes(bytes);
                        }
                    }
                    0x7F => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_shadow_has_alloc =
                                u64::from_le_bytes(bytes);
                        }
                    }
                    0x80 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_shadow_alloc_val =
                                u64::from_le_bytes(bytes);
                        }
                    }
                    0x81 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_shadow_up = u64::from_le_bytes(bytes);
                        }
                    }
                    0x82 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_shadow_down = u64::from_le_bytes(bytes);
                        }
                    }
                    0x83 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_shadow_up_all = u64::from_le_bytes(bytes);
                        }
                    }
                    0x84 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_shadow_down_all = u64::from_le_bytes(bytes);
                        }
                    }
                    0x85 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_shadow_num_allocs =
                                u64::from_le_bytes(bytes);
                        }
                    }
                    0x86 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_shadow_allocs_sum =
                                u64::from_le_bytes(bytes);
                        }
                    }
                    0x87 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_ext_balance = u64::from_le_bytes(bytes);
                        }
                    }
                    0x88 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_self_balance = u64::from_le_bytes(bytes);
                        }
                    }
                    0x89 => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_transfer = u64::from_le_bytes(bytes);
                        }
                    }
                    0x8A => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_update_param = u64::from_le_bytes(bytes);
                        }
                    }
                    0x8B => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_gov_account = u64::from_le_bytes(bytes);
                        }
                    }
                    0x8C => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_gov_contract = u64::from_le_bytes(bytes);
                        }
                    }
                    0x8D => {
                        if let Ok(bytes) = value.as_ref().try_into() {
                            params_holder.opcode_ops.op_reconstitute = u64::from_le_bytes(bytes);
                        }
                    }
                    _ => (),
                },
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
        self.get_mut_ephemeral_params_holder()
            .move_ppm_liquidity_fee = value;
    }
    pub fn set_in_call_ppm_liquidity_fee(&mut self, value: u64) {
        self.get_mut_ephemeral_params_holder()
            .in_call_ppm_liquidity_fee = value;
    }
    pub fn set_swapout_entry_base_fee(&mut self, value: u64) {
        self.get_mut_ephemeral_params_holder()
            .swapout_entry_base_fee = value;
    }
    pub fn set_config_entry_base_fee(&mut self, value: u64) {
        self.get_mut_ephemeral_params_holder().config_entry_base_fee = value;
    }
    pub fn set_config_entry_per_config_byte_fee(&mut self, value: u64) {
        self.get_mut_ephemeral_params_holder()
            .config_entry_per_config_byte_fee = value;
    }
    pub fn set_deploy_entry_base_fee(&mut self, value: u64) {
        self.get_mut_ephemeral_params_holder().deploy_entry_base_fee = value;
    }
    pub fn set_deploy_entry_per_program_byte_fee(&mut self, value: u64) {
        self.get_mut_ephemeral_params_holder()
            .deploy_entry_per_program_byte_fee = value;
    }

    /// Reverts the ephemeral changes associated with the last execution.
    pub fn rollback_last(&mut self) {
        self.restore_delta();
    }

    /// Applies all ephemeral changes from delta into permanent in-memory and on-disk state.
    pub fn apply_changes(&mut self) -> Result<(), sled::Error> {
        if let Some(ephemeral_params_holder) = self.delta.updated_params_holder.as_ref() {
            let tree = self.on_disk_params.open_tree(PARAMS_HOLDER_TREE_NAME)?;

            tree.insert(
                ACCOUNT_CAN_INITIALLY_DEPLOY_LIQUIDITY_SPECIAL_DB_KEY,
                [
                    if ephemeral_params_holder.account_can_initially_deploy_liquidity {
                        1u8
                    } else {
                        0u8
                    },
                ]
                .as_slice(),
            )?;
            tree.insert(
                ACCOUNT_CAN_INITIALLY_DEPLOY_CONTRACT_SPECIAL_DB_KEY,
                [
                    if ephemeral_params_holder.account_can_initially_deploy_contract {
                        1u8
                    } else {
                        0u8
                    },
                ]
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
            tree.insert(
                SWAPOUT_ENTRY_BASE_FEE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .swapout_entry_base_fee
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                CONFIG_ENTRY_BASE_FEE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .config_entry_base_fee
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                CONFIG_ENTRY_PER_CONFIG_BYTE_FEE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .config_entry_per_config_byte_fee
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                DEPLOY_ENTRY_BASE_FEE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .deploy_entry_base_fee
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                DEPLOY_ENTRY_PER_PROGRAM_BYTE_FEE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .deploy_entry_per_program_byte_fee
                    .to_le_bytes()
                    .to_vec(),
            )?;

            tree.insert(
                OP_FALSE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_false
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_TRUE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_true
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_2_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_2
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_3_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_3
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_4_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_4
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_5_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_5
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_6_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_6
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_7_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_7
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_8_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_8
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_9_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_9
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_10_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_10
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_11_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_11
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_12_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_12
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_13_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_13
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_14_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_14
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_15_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_15
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_16_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_16
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_PUSHDATA_BASE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_pushdata_base
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_PUSHDATA_PER_BYTE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_pushdata_per_byte
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_NOP_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_nop
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_JUMP_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_jump
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_IF_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_if
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_NOTIF_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_notif
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_ELSE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_else
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_ENDIF_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_endif
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_VERIFY_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_verify
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_RETURNALL_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_returnall
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_RETURNSOME_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_returnsome
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_FAIL_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_fail
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_RETURNERR_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_returnerr
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_TOALTSTACK_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_toaltstack
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_FROMALTSTACK_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_fromaltstack
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_2DROP_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_2drop
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_2DUP_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_2dup
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_3DUP_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_3dup
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_2OVER_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_2over
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_2ROT_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_2rot
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_2SWAP_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_2swap
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_IFDUP_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_ifdup
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_DEPTH_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_depth
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_DROP_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_drop
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_DUP_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_dup
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_NIP_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_nip
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_OVER_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_over
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_PICK_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_pick
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_ROLL_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_roll
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_ROT_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_rot
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_SWAP_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_swap
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_TUCK_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_tuck
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_CAT_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_cat
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_SPLIT_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_split
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_LEFT_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_left
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_RIGHT_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_right
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_SIZE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_size
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_INVERT_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_invert
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_AND_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_and
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_OR_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_or
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_XOR_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_xor
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_EQUAL_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_equal
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_EQUALVERIFY_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_equalverify
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_REVERSE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_reverse
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_1ADD_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_1add
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_1SUB_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_1sub
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_2MUL_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_2mul
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_2DIV_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_2div
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_ADDMOD_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_addmod
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_MULMOD_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_mulmod
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_NOT_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_not
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_0NOTEQUAL_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_0notequal
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_ADD_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_add
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_SUB_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_sub
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_MUL_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_mul
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_DIV_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_div
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_LSHIFT_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_lshift
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_RSHIFT_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_rshift
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_BOOLAND_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_booland
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_BOOLOR_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_boolor
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_NUMEQUAL_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_numequal
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_NUMEQUALVERIFY_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_numequalverify
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_NUMNOTEQUAL_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_numnotequal
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_LESSTHAN_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_lessthan
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_GREATERTHAN_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_greaterthan
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_LESSTHANOREQUAL_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_lessthanorequal
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_GREATERTHANOREQUAL_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_greaterthanorequal
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_MIN_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_min
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_MAX_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_max
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_WITHIN_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_within
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_MREAD_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_mread
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_MWRITE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_mwrite
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_MFREE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_mfree
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_RIPEMD160_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_ripemd160
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_SHA1_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_sha1
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_SHA256_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_sha256
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_HASH160_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_hash160
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_HASH256_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_hash256
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_TAGGEDHASH_BASE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_taggedhash_base
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_TAGGEDHASH_PER_BYTE_GAP_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_taggedhash_per_byte_gap
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_TAGGEDHASH_OUTPUT_LEN_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_taggedhash_output_len
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_BLAKE2BVAR_BASE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_blake2bvar_base
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_BLAKE2BVAR_PER_BYTE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_blake2bvar_per_byte
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_BLAKE2SVAR_BASE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_blake2svar_base
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_BLAKE2SVAR_PER_BYTE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_blake2svar_per_byte
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_SECPSCALARADD_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_secpscalaradd
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_SECPSCALARMUL_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_secpscalarmul
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_SECPPOINTADD_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_secppointadd
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_SECPPOINTMUL_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_secppointmul
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_PUSHSECPGENERATORPOINT_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_pushsecpgeneratorpoint
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_ISZEROSECPSCALAR_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_iszerosecpscalar
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_ISINFINITESECPPOINT_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_isinfinitesecppoint
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_CHECKSCHNORRSIG_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_checkschnorrsig
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_CHECKSCHNORRSIGBIP340_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_checkschnorrsigbip340
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_CHECKBLSSIG_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_checkblssig
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_CHECKBLSSIGAGG_BASE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_checkblssigagg_base
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_CHECKBLSSIGAGG_PER_COUNT_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_checkblssigagg_per_count
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_CALLER_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_caller
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_OPSBUDGET_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_opsbudget
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_OPSCOUNTER_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_opscounter
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_OPSPRICE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_opsprice
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_TIMESTAMP_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_timestamp
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_CALL_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_call
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_CALLEXT_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_callext
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_SREAD_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_sread
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_SFREE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_sfree
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_SWRITE_BASE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_swrite_base
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_SWRITE_PER_BYTE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_swrite_per_byte
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_SHADOW_ALLOC_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_shadow_alloc
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_SHADOW_DEALLOC_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_shadow_dealloc
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_SHADOW_HAS_ALLOC_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_shadow_has_alloc
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_SHADOW_ALLOC_VAL_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_shadow_alloc_val
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_SHADOW_UP_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_shadow_up
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_SHADOW_DOWN_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_shadow_down
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_SHADOW_UP_ALL_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_shadow_up_all
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_SHADOW_DOWN_ALL_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_shadow_down_all
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_SHADOW_NUM_ALLOCS_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_shadow_num_allocs
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_SHADOW_ALLOCS_SUM_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_shadow_allocs_sum
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_EXT_BALANCE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_ext_balance
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_SELF_BALANCE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_self_balance
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_TRANSFER_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_transfer
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_UPDATE_PARAM_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_update_param
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_GOV_ACCOUNT_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_gov_account
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_GOV_CONTRACT_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_gov_contract
                    .to_le_bytes()
                    .to_vec(),
            )?;
            tree.insert(
                OP_RECONSTITUTE_SPECIAL_DB_KEY,
                ephemeral_params_holder
                    .opcode_ops
                    .op_reconstitute
                    .to_le_bytes()
                    .to_vec(),
            )?;

            self.in_memory_params_holder = ephemeral_params_holder.clone();
        }

        Ok(())
    }

    /// Clears all ephemeral changes from delta and backup.
    pub fn flush_delta(&mut self) {
        self.delta.flush();
        self.backup_of_delta.flush();
    }
}

/// Erases the params manager by db path.
pub fn erase_params_manager(chain: Chain) {
    let params_db_path = format!("storage/{}/params", chain);
    let _ = std::fs::remove_dir_all(params_db_path);
}
