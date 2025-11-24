use crate::{
    constructive::{txn::ext::OutpointExt, txo::zktlc::ZKTLC},
    operative::Chain,
};
use secp::Point;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

/// Owner key of a ZKTLC.
type AccountKey = Point;

/// Guarded ZKTLC set.
#[allow(non_camel_case_types)]
pub type ZKTLC_SET = Arc<Mutex<ZKTLCSet>>;

/// A lookup struct for storing ZKTLCs.
pub struct ZKTLCSet {
    // In-memory ZKTLC set.
    zktlcs: HashMap<AccountKey, Vec<ZKTLC>>,
    // In-storage ZKTLC set.
    zktlcs_db: sled::Db,
}

impl ZKTLCSet {
    /// Creates the ZKTLCSet instance.
    pub fn new(chain: Chain) -> Option<ZKTLC_SET> {
        // Collect VTXOs from db.
        let zktlcs_path = format!("{}/{}/{}", "storage", chain.to_string(), "set/zktlc");
        let zktlcs_db = sled::open(zktlcs_path).ok()?;

        let mut global_zktlc_set = HashMap::<AccountKey, Vec<ZKTLC>>::new();

        // Load VTXOs from db.
        for lookup in zktlcs_db.iter() {
            if let Ok((_, val)) = lookup {
                // Deserialize VTXO.
                let zktlc = serde_json::from_slice::<ZKTLC>(&val).ok()?;

                // Get account key.
                let account_key = zktlc.account_key();

                // Get account VTXO set.
                let account_zktlc_set = match global_zktlc_set.get_mut(&account_key) {
                    Some(set) => set,
                    None => {
                        // Create empty set if not exists and return it.
                        let empty_set = Vec::<ZKTLC>::new();
                        global_zktlc_set.insert(account_key, empty_set);
                        global_zktlc_set.get_mut(&account_key)?
                    }
                };

                // Insert VTXO to the account's VTXO set.
                account_zktlc_set.push(zktlc);
            }
        }

        // Construct ZKTLCSet instance.
        let zktlc_set = ZKTLCSet {
            zktlcs: global_zktlc_set,
            zktlcs_db,
        };

        // Return the ZKTLCSet instance.
        Some(Arc::new(Mutex::new(zktlc_set)))
    }

    /// Returns the ZKTLC set of a given account key.
    pub fn zktlc_set_by_account_key(&self, account_key: &Point) -> Vec<ZKTLC> {
        self.zktlcs
            .get(account_key)
            .map(|zktlcs| zktlcs.clone())
            .unwrap_or(Vec::<ZKTLC>::new())
    }

    /// Inserts a ZKTLC to the ZKTLC set.
    pub fn insert_zktlc(&mut self, zktlc: &ZKTLC) -> bool {
        // Get ZKTLC's account key.
        let account_key = zktlc.account_key();

        // Get ZKTLC's outpoint.
        let zktlc_outpoint = match zktlc.outpoint() {
            Some(outpoint) => outpoint,
            None => return false,
        };

        // Check if ZKTLC has a rollup height.
        if let None = zktlc.at_rollup_height() {
            return false;
        }

        // TODO: Check if ZKTLC has a bitcoin height. (maybe?)

        // Return the account's ZKTLC set.
        let account_zktlc_set = match self.zktlcs.get_mut(&account_key) {
            Some(set) => set,
            None => {
                let empty_set = Vec::<ZKTLC>::new();
                self.zktlcs.insert(account_key, empty_set);
                match self.zktlcs.get_mut(&account_key) {
                    Some(set) => set,
                    None => return false,
                }
            }
        };

        // Check if the ZKTLC already exists.
        if account_zktlc_set
            .iter()
            .any(|account_zktlc| account_zktlc.outpoint() == Some(zktlc_outpoint))
        {
            return false;
        }

        // Insert ZKTLC to the in-memory set.
        account_zktlc_set.push(zktlc.to_owned());

        // Insert ZKTLC to the in-storage set.
        match self
            .zktlcs_db
            .insert(&zktlc_outpoint.bytes_36(), zktlc.serialize())
        {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    // Removes a ZKTLC from the ZKTLC set.
    pub fn remove_zktlc(&mut self, zktlc: &ZKTLC) -> bool {
        // Get ZKTLC's account key.
        let account_key = zktlc.account_key();

        // Get ZKTLC's outpoint.
        let zktlc_outpoint = match zktlc.outpoint() {
            Some(outpoint) => outpoint,
            None => return false,
        };

        // Return the account's ZKTLC set.
        let account_zktlc_set = match self.zktlcs.get_mut(&account_key) {
            Some(set) => set,
            None => return false,
        };

        // Remove ZKTLC from the in-memory set.
        account_zktlc_set.retain(|zktlc| zktlc.outpoint() != Some(zktlc_outpoint));

        // Remove ZKTLC from the in-storage set.
        match self.zktlcs_db.remove(&zktlc_outpoint.bytes_36()) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Returns the ZKTLCs to recharge.
    pub fn zktlcs_to_recharge(
        &self,
        account_key: &Point,
        current_bitcoin_height: u32,
    ) -> Vec<ZKTLC> {
        // Retrieve the account's ZKTLC set.
        let account_zktlc_set = self.zktlc_set_by_account_key(account_key);

        // Filter the ZKTLCs that are rechargeable.
        let rechargeable_zktlcs = account_zktlc_set
            .iter()
            .filter(|zktlc| {
                zktlc
                    .is_rechargeable(current_bitcoin_height)
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        // Return the rechargeable ZKTLCs.
        rechargeable_zktlcs
    }
}
