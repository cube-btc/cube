use crate::{
    inscriptive::sync_manager::errors::construction_error::SMConstructionError,
    operative::run_args::chain::Chain,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// A struct for managing the sync heights of the Bitcoin and batch.
pub struct SyncManager {
    synced: bool,
    // Bitcoin sync height.
    bitcoin_sync_height: u64,

    // Batch sync height.
    batch_sync_height: u64,

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

        // 2 Collect the sync heights from the db.
        let bitcoin_sync_height: u64 = db
            .get(b"bitcoin_sync_height")
            .ok()
            .flatten()
            .and_then(|val| val.as_ref().try_into().ok().map(u64::from_be_bytes))
            .unwrap_or(0);

        // 3 Collect the batch sync height from the db.
        let batch_sync_height: u64 = db
            .get(b"batch_sync_height")
            .ok()
            .flatten()
            .and_then(|val| val.as_ref().try_into().ok().map(u64::from_be_bytes))
            .unwrap_or(0);

        // 4 Construct the sync manager.
        let sync_manager = SyncManager {
            synced: false,
            bitcoin_sync_height,
            batch_sync_height,
            db,
        };

        // 5 Guard the sync manager.
        let sync_manager = Arc::new(Mutex::new(sync_manager));

        // 6 Return the sync manager.
        Ok(sync_manager)
    }

    /// Sets the synced flag.
    pub fn set_synced(&mut self, synced: bool) {
        self.synced = synced;
    }

    pub fn is_synced(&self) -> bool {
        self.synced
    }

    /// Returns the bitcoin sync height tip.
    pub fn bitcoin_sync_height_tip(&self) -> u64 {
        self.bitcoin_sync_height
    }

    /// Returns the cube batch sync height tip.
    pub fn cube_batch_sync_height_tip(&self) -> u64 {
        self.batch_sync_height
    }

    /// Sets the bitcoin sync height.
    pub fn set_bitcoin_sync_height(&mut self, height: u64) {
        // Update in-memory.
        self.bitcoin_sync_height = height;

        // Update in-db.
        let _ = self
            .db
            .insert(b"bitcoin_sync_height", height.to_be_bytes().to_vec());
    }

    /// Sets the batch sync height.
    pub fn set_batch_sync_height(&mut self, height: u64) {
        // Update in-memory.
        self.batch_sync_height = height;

        // Update in-db.
        let _ = self
            .db
            .insert(b"batch_sync_height", height.to_be_bytes().to_vec());
    }
}

/// Erases the sync manager by db path.
pub fn erase_sync_manager(chain: Chain) {
    // Sync manager db path.
    let sync_manager_db_path = format!("storage/{}/sync_manager", chain.to_string());

    // Erase the sync manager db path.
    let _ = std::fs::remove_dir_all(sync_manager_db_path);
}