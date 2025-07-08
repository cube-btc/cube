use crate::{
    constructive::{txn::ext::OutpointExt, txo::vtxo::VTXO},
    operative::Chain,
};
use secp::Point;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

/// Owner key of a VTXO.
type AccountKey = Point;

/// Guarded VTXO set.
#[allow(non_camel_case_types)]
pub type VTXO_SET = Arc<Mutex<VTXOSet>>;

/// A lookup struct for storing virtual UTXOs (VTXOs).
pub struct VTXOSet {
    // In-memory VTXO set.
    vtxos: HashMap<AccountKey, Vec<VTXO>>,
    // In-storage VTXO set.
    vtxos_db: sled::Db,
}

impl VTXOSet {
    /// Creates the VTXOSet instance.
    pub fn new(chain: Chain) -> Option<VTXO_SET> {
        // Collect VTXOs from db.
        let vtxos_path = format!("{}/{}/{}", "db", chain.to_string(), "set/vtxo");
        let vtxos_db = sled::open(vtxos_path).ok()?;

        let mut global_vtxo_set = HashMap::<AccountKey, Vec<VTXO>>::new();

        // Load VTXOs from db.
        for lookup in vtxos_db.iter() {
            if let Ok((_, val)) = lookup {
                // Deserialize VTXO.
                let vtxo = serde_json::from_slice::<VTXO>(&val).ok()?;

                // Get account key.
                let account_key = vtxo.account_key();

                // Get account VTXO set.
                let account_vtxo_set = match global_vtxo_set.get_mut(&account_key) {
                    Some(set) => set,
                    None => {
                        // Create empty set if not exists and return it.
                        let empty_set = Vec::<VTXO>::new();
                        global_vtxo_set.insert(account_key, empty_set);
                        global_vtxo_set.get_mut(&account_key)?
                    }
                };

                // Insert VTXO to the account's VTXO set.
                account_vtxo_set.push(vtxo);
            }
        }

        // Construct VTXOSet instance.
        let vtxoset = VTXOSet {
            vtxos: global_vtxo_set,
            vtxos_db,
        };

        // Return the VTXOSet instance.
        Some(Arc::new(Mutex::new(vtxoset)))
    }

    /// Returns the VTXO set of a given account key.
    pub fn vtxo_set_by_account_key(&self, account_key: &Point) -> Vec<VTXO> {
        self.vtxos
            .get(account_key)
            .map(|vtxos| vtxos.clone())
            .unwrap_or(Vec::<VTXO>::new())
    }

    /// Inserts a VTXO to the VTXO set.
    pub fn insert_vtxo(&mut self, vtxo: &VTXO) -> bool {
        // Get VTXO's account key.
        let account_key = vtxo.account_key();

        // Get VTXO's outpoint.
        let vtxo_outpoint = match vtxo.outpoint() {
            Some(outpoint) => outpoint,
            None => return false,
        };

        // Check if VTXO has a rollup height.
        if let None = vtxo.at_rollup_height() {
            return false;
        }

        // TODO: Check if VTXO has a bitcoin height. (maybe?)

        // Return the account's VTXO set.
        let account_vtxo_set = match self.vtxos.get_mut(&account_key) {
            Some(set) => set,
            None => {
                let empty_set = Vec::<VTXO>::new();
                self.vtxos.insert(account_key, empty_set);
                match self.vtxos.get_mut(&account_key) {
                    Some(set) => set,
                    None => return false,
                }
            }
        };

        // Check if the VTXO already exists.
        if account_vtxo_set
            .iter()
            .any(|account_vtxo| account_vtxo.outpoint() == Some(vtxo_outpoint))
        {
            return false;
        }

        // Insert VTXO to the in-memory set.
        account_vtxo_set.push(vtxo.to_owned());

        // Insert VTXO to the in-storage set.
        match self
            .vtxos_db
            .insert(&vtxo_outpoint.bytes_36(), vtxo.serialize())
        {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    // Removes a VTXO from the VTXO set.
    pub fn remove_vtxo(&mut self, vtxo: &VTXO) -> bool {
        // Get VTXO's account key.
        let account_key = vtxo.account_key();

        // Get VTXO's outpoint.
        let vtxo_outpoint = match vtxo.outpoint() {
            Some(outpoint) => outpoint,
            None => return false,
        };

        // Return the account's VTXO set.
        let account_vtxo_set = match self.vtxos.get_mut(&account_key) {
            Some(set) => set,
            None => return false,
        };

        // Remove VTXO from the in-memory set.
        account_vtxo_set.retain(|vtxo| vtxo.outpoint() != Some(vtxo_outpoint));

        // Remove VTXO from the in-storage set.
        match self.vtxos_db.remove(&vtxo_outpoint.bytes_36()) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Returns the VTXOs to recharge.
    pub fn vtxos_to_recharge(&self, account_key: &Point, current_bitcoin_height: u32) -> Vec<VTXO> {
        // Retrieve the account's VTXO set.
        let account_vtxo_set = self.vtxo_set_by_account_key(account_key);

        // Filter the VTXOs that are rechargeable.
        let rechargeable_vtxos = account_vtxo_set
            .iter()
            .filter(|vtxo| {
                vtxo.is_rechargeable(current_bitcoin_height)
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        // Return the rechargeable VTXOs.
        rechargeable_vtxos
    }
}
