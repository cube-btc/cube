use crate::{
    constructive::txn::ext::{OutpointExt, TxOutExt},
    operative::Chain,
};
use bitcoin::{OutPoint, TxOut};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

/// Guarded UTXO set.
#[allow(non_camel_case_types)]
pub type UTXO_SET = Arc<Mutex<UTXOSet>>;

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

    /// Inserts a txout into the set.
    pub fn insert_txout(&mut self, outpoint: &OutPoint, txout: &TxOut) {
        // Insert txout into the in-memory set.
        if let None = self.utxos.insert(outpoint.clone(), txout.clone()) {
            // Insert txout into the in-storage set.
            let _ = self.utxos_db.insert(&outpoint.bytes_36(), txout.bytes());
        }
    }

    /// Removes a txout from the set.
    pub fn remove_txout(&mut self, outpoint: &OutPoint) {
        // Remove txout from the in-memory set.
        if let Some(_) = self.utxos.remove(outpoint) {
            // Remove txout from the in-storage set.
            let _ = self.utxos_db.remove(&outpoint.bytes_36());
        }
    }
}
