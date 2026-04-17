use crate::{
    constructive::txout_types::payload::payload::{genesis_payload, Payload},
    inscriptive::sync_manager::errors::construction_error::SMConstructionError,
    operative::run_args::chain::Chain,
};
use bitcoin::hashes::Hash;
use std::sync::Arc;
use tokio::sync::Mutex;

/// A struct for managing the sync tips of the Bitcoin and cube batch.
pub struct SyncManager {
    // Synced flag.
    synced: bool,

    // Bitcoin sync height tip.
    bitcoin_sync_height_tip: u64,

    // Cube batch sync height tip.
    cube_batch_sync_height_tip: u64,

    // Payload tip.
    payload_tip: Payload,

    // In-storage db.
    db: sled::Db,
}

/// Guarded sync manager.
#[allow(non_camel_case_types)]
pub type SYNC_MANAGER = Arc<Mutex<SyncManager>>;

impl SyncManager {
    pub fn new(chain: Chain) -> Result<SYNC_MANAGER, SMConstructionError> {
        // 1 Open the sync manager db.
        let db_path = format!("storage/{}/sync_manager", chain.to_string());
        let db = sled::open(db_path).map_err(SMConstructionError::DBOpenError)?;

        // 2 Get the bitcoin sync height tip from the db.
        let bitcoin_sync_height_tip: u64 = db
            .get(b"bitcoin_sync_height_tip")
            .ok()
            .flatten()
            .and_then(|val| val.as_ref().try_into().ok().map(u64::from_be_bytes))
            .unwrap_or(0);

        // 3 Get the cube batch sync height tip from the db.
        let cube_batch_sync_height_tip: u64 = db
            .get(b"cube_batch_sync_height_tip")
            .ok()
            .flatten()
            .and_then(|val| val.as_ref().try_into().ok().map(u64::from_be_bytes))
            .unwrap_or(0);

        // 4 Get the payload tip from the db.
        let payload_tip: Payload = {
            db.get(b"payload_tip")
                .ok()
                .flatten()
                .or_else(|| db.get(b"prev_payload").ok().flatten())
                .and_then(|payload_bytes| Payload::deserialize(payload_bytes.as_ref()))
                .unwrap_or_else(|| genesis_payload(chain))
        };

        // 5 Construct the sync manager.
        let sync_manager = SyncManager {
            synced: false,
            bitcoin_sync_height_tip,
            cube_batch_sync_height_tip,
            payload_tip,
            db,
        };

        // 6 Guard the sync manager.
        let sync_manager = Arc::new(Mutex::new(sync_manager));

        // 7 Return the sync manager.
        Ok(sync_manager)
    }

    /// Sets the synced flag.
    pub fn set_synced(&mut self, synced: bool) {
        self.synced = synced;
    }

    /// Returns the synced flag.
    pub fn is_synced(&self) -> bool {
        self.synced
    }

    /// Returns the bitcoin sync height tip.
    pub fn bitcoin_sync_height_tip(&self) -> u64 {
        self.bitcoin_sync_height_tip
    }

    /// Returns the cube batch sync height tip.
    pub fn cube_batch_sync_height_tip(&self) -> u64 {
        self.cube_batch_sync_height_tip
    }

    /// Returns the cube batch transaction id tip.
    pub fn cube_batch_tx_id_tip(&self) -> [u8; 32] {
        self.payload_tip
            .location()
            .map(|(outpoint, _)| outpoint.txid)
            .expect("Payload tip location must exist to get cube batch tx id tip")
            .to_raw_hash()
            .to_byte_array()
    }

    /// Returns the payload tip.
    pub fn payload_tip(&self) -> Payload {
        self.payload_tip.clone()
    }

    /// Sets the bitcoin sync height tip.
    pub fn set_bitcoin_sync_height_tip(&mut self, height: u64) {
        // Update in-memory.
        self.bitcoin_sync_height_tip = height;

        // Update in-db.
        let _ = self
            .db
            .insert(b"bitcoin_sync_height_tip", height.to_be_bytes().to_vec());
    }

    /// Sets the cube batch sync height tip.
    pub fn set_cube_batch_sync_height_tip(&mut self, height: u64) {
        // Update in-memory.
        self.cube_batch_sync_height_tip = height;

        // Update in-db.
        let _ = self
            .db
            .insert(b"cube_batch_sync_height_tip", height.to_be_bytes().to_vec());
    }

    /// Sets the payload tip.
    pub fn set_payload_tip(&mut self, payload_tip: Payload) {
        // Update in-memory.
        self.payload_tip = payload_tip.clone();

        // Update in-db.
        if let Some(payload_bytes) = payload_tip.serialize() {
            let _ = self.db.insert(b"payload_tip", payload_bytes);
        }
    }
}

/// Erases the sync manager by db path.
pub fn erase_sync_manager(chain: Chain) {
    // Sync manager db path.
    let sync_manager_db_path = format!("storage/{}/sync_manager", chain.to_string());

    // Erase the sync manager db path.
    let _ = std::fs::remove_dir_all(sync_manager_db_path);
}
