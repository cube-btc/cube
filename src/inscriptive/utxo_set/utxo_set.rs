use crate::{
    constructive::{
        txn::ext::{OutpointExt, TxOutExt},
        txo::lift::{
            lift::Lift,
            lift_versions::{
                liftv1::return_liftv1_scriptpubkey, liftv2::return_liftv2_scriptpubkey,
            },
        },
    },
    operative::Chain,
};
use bitcoin::{OutPoint, TxOut};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

/// A lookup struct for storing bare UTXOs.
/// Bitcoin blocks are synced from the network, and the UTXO set is constructed by scanning each block.
/// The purpose of this set is to provide a quick lookup for `TxHolder` to locate `Lift` prevouts.
///
/// For storage efficiency, this set does not include Bitcoin's entire UTXO set but only those created after `SYNC_START_HEIGHT`,
/// as no `Lift` outputs were created before this height.
///
/// Since the connected Bitcoin RPC node already maintains the entire UTXO set,
/// this set is optimized solely for quick lookup of `Lift` prevouts by the Cube nodes.
///
pub struct UTXOSet {
    // In-memory UTXO set.
    utxos: HashMap<OutPoint, TxOut>,

    // In-storage UTXO set.
    utxos_db: sled::Db,
}

/// Guarded UTXO set.
#[allow(non_camel_case_types)]
pub type UTXO_SET = Arc<Mutex<UTXOSet>>;

impl UTXOSet {
    /// Creates the UTXOSet instance.
    pub fn new(chain: Chain) -> Option<UTXO_SET> {
        // Collect UTXOs from db.
        let utxos_path = format!("{}/{}/{}", "storage", chain.to_string(), "set/utxo");
        let utxos_db = sled::open(utxos_path).ok()?;

        let mut utxos = HashMap::<OutPoint, TxOut>::new();

        // Load UTXOs from db.
        for lookup in utxos_db.iter() {
            if let Ok((key, val)) = lookup {
                // Deserialize outpoint.
                let outpoint_bytes: [u8; 36] = key.as_ref().try_into().ok()?;
                let outpoint = OutPoint::from_bytes36(&outpoint_bytes)?;

                // Deserialize txout.
                let txout = TxOut::from_bytes(val.as_ref())?;

                // Insert utxo.
                utxos.insert(outpoint, txout);
            }
        }

        // Construct the UTXOSet instance.
        let utxoset = UTXOSet { utxos, utxos_db };

        // Return the UTXOSet instance.
        Some(Arc::new(Mutex::new(utxoset)))
    }

    /// Returns the number of utxos in the set.
    pub fn num_utxos(&self) -> usize {
        self.utxos.len()
    }

    /// Returns the utxo at the given script pubkey.
    pub fn txout_by_outpoint(&self, outpoint: &OutPoint) -> Option<TxOut> {
        self.utxos.get(outpoint).cloned()
    }

    /// Inserts a utxo into the set.
    pub fn insert_utxo(&mut self, outpoint: &OutPoint, txout: &TxOut) {
        // Insert utxo into the in-memory set.
        if let None = self.utxos.insert(outpoint.clone(), txout.clone()) {
            // Insert utxo into the in-storage set.
            let _ = self.utxos_db.insert(&outpoint.bytes_36(), txout.bytes());
        }
    }

    /// Removes a utxo from the set.
    pub fn remove_utxo(&mut self, outpoint: &OutPoint) {
        // Remove utxo from the in-memory set.
        if let Some(_) = self.utxos.remove(outpoint) {
            // Remove utxo from the in-storage set.
            let _ = self.utxos_db.remove(&outpoint.bytes_36());
        }
    }

    /// Returns UTXOs in this set whose script pubkey matches self owned `Lift` transaction outputs.
    ///
    /// Used by an `Account` to scan the UTXO set and collect the self owned `Lift`s.
    pub fn scan_and_return_self_owned_lifts(
        &self,
        engine_key: &[u8; 32],
        self_account_key: &[u8; 32],
        v2_interactive_enabled: bool,
    ) -> Vec<Lift> {
        // 1 Acquire the account and engine keys.
        let account_key = *self_account_key;
        let engine_key = *engine_key;

        // 2 Construct the script pubkeys.
        let spk_v1 = return_liftv1_scriptpubkey(account_key, engine_key);
        let spk_v2 = if v2_interactive_enabled {
            return_liftv2_scriptpubkey(account_key, engine_key)
        } else {
            None
        };

        // 3 Initialize the lifts vector.
        let mut lifts = Vec::new();

        // 4 Scan the UTXOs in the set and collect the self owned `Lift`s.
        for (outpoint, txout) in &self.utxos {
            let spk = txout.script_pubkey.as_bytes();

            if let Some(ref expected) = spk_v1 {
                if spk == expected.as_slice() {
                    lifts.push(Lift::new_liftv1(
                        account_key,
                        engine_key,
                        outpoint.clone(),
                        txout.clone(),
                    ));

                    continue;
                }
            }

            if let Some(ref expected) = spk_v2 {
                if spk == expected.as_slice() {
                    lifts.push(Lift::new_liftv2(
                        account_key,
                        engine_key,
                        outpoint.clone(),
                        txout.clone(),
                    ));
                }
            }
        }

        // 5 Return the lifts.
        lifts
    }

    /// Validates the `Lift` prevouts in the `Liftup`.
    ///
    /// Used by the `Engine` to validate the `Lift`s in a `Liftup` are indeed valid UTXOs.
    pub fn validate_lifts(&self, lifts_to_validate: &Vec<Lift>) -> bool {
        lifts_to_validate.iter().all(|lift| {
            self.utxos.iter().any(|(existing_outpoint, txout)| {
                existing_outpoint == &lift.outpoint()
                    && txout.value.to_sat() == lift.txout().value.to_sat()
            })
        })
    }
}
