use crate::constructive::bitcoiny::batch_container::batch_container::BatchContainer;
use crate::constructive::bitcoiny::batch_record::batch_record::BatchRecord;
use crate::constructive::entry::entry::entry::Entry;
use crate::inscriptive::archival_manager::errors::construction_error::ArchivalConstructionError;
use crate::inscriptive::archival_manager::errors::insert_error::ArchivalManagerInsertBatchRecordError;
use crate::operative::run_args::chain::Chain;
use bitcoin::hashes::Hash;
use bitcoin::OutPoint;
use bitcoin::Txid;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Type alias for the batch height.
pub type BatchHeight = u64;

/// Type alias for the batch Bitcoin transaction id.
pub type BatchTxid = [u8; 32];

/// Type alias for the batch timestamp.
pub type BatchTimestamp = u64;

/// Type alias for the entry id.
pub type EntryId = [u8; 32];

/// Local storage manager for `BatchRecord` for nodes that run in archival mode.
pub struct ArchivalManager {
    // In-memory batch records keyed by batch height.
    in_memory_records: HashMap<BatchHeight, BatchRecord>,

    // On-disk batch records.
    in_db_records: sled::Db,
}

/// Guarded `ArchivalManager`.
#[allow(non_camel_case_types)]
pub type ARCHIVAL_MANAGER = Arc<Mutex<ArchivalManager>>;

/// Returns whether `entry` is attributed to `account_key` (secp x-only account id).
fn entry_involves_account(entry: &Entry, account_key: [u8; 32]) -> bool {
    match entry {
        Entry::Move(move_entry) => {
            move_entry.from.account_key() == account_key || move_entry.to.account_key() == account_key
        }
        Entry::Call(call) => call.account.account_key() == account_key,
        Entry::Liftup(liftup) => liftup.root_account.account_key() == account_key,
    }
}

/// Batch heights present in memory, ascending (stable scan order).
fn sorted_batch_heights(map: &HashMap<BatchHeight, BatchRecord>) -> Vec<BatchHeight> {
    let mut heights: Vec<BatchHeight> = map.keys().copied().collect();
    heights.sort_unstable();
    heights
}

impl ArchivalManager {
    /// Constructs an `ArchivalManager` by opening storage and loading existing `BatchRecord`s.
    pub fn new(chain: Chain) -> Result<ARCHIVAL_MANAGER, ArchivalConstructionError> {
        // 1 Open the archival manager db.
        let db_path = format!("storage/{}/archival_manager", chain.to_string());
        let in_db_records = sled::open(&db_path).map_err(ArchivalConstructionError::DBOpenError)?;

        // 2 Initialize the in-memory map of loaded records.
        let mut loaded: HashMap<BatchHeight, BatchRecord> = HashMap::new();

        // 3 Iterate all key-value pairs in the db.
        for item in in_db_records.iter().filter_map(|r| r.ok()) {
            // 3.1 Get the key and value.
            let (k, v) = item;

            // 3.2 Require an 8-byte batch height key.
            if k.len() != 8 {
                return Err(ArchivalConstructionError::UnexpectedDbKeyLength(k.len()));
            }

            // 3.3 Decode the batch height from the key.
            let key_bytes: [u8; 8] = k
                .as_ref()
                .try_into()
                .map_err(|_| ArchivalConstructionError::UnexpectedDbKeyLength(k.len()))?;
            let height = u64::from_be_bytes(key_bytes);

            // 3.4 Deserialize the batch record.
            let record = BatchRecord::deserialize(v.as_ref())
                .ok_or(ArchivalConstructionError::CorruptRecord(height))?;

            // 3.5 Ensure the record's batch height matches the db key.
            if record.batch_height != height {
                return Err(ArchivalConstructionError::HeightKeyMismatch {
                    db_key_height: height,
                    record_batch_height: record.batch_height,
                });
            }

            // 3.6 Insert the record keyed by batch height (reject duplicate heights in db).
            match loaded.entry(record.batch_height) {
                std::collections::hash_map::Entry::Vacant(slot) => {
                    slot.insert(record);
                }
                std::collections::hash_map::Entry::Occupied(_) => {
                    return Err(ArchivalConstructionError::CorruptRecord(height));
                }
            }
        }

        // 4 Construct the archival manager.
        let manager = ArchivalManager {
            in_memory_records: loaded,
            in_db_records,
        };

        // 5 Guard the archival manager.
        let manager = Arc::new(Mutex::new(manager));

        // 6 Return the guarded archival manager.
        Ok(manager)
    }

    /// Inserts a new `BatchRecord`. Returns an error if that batch height already exists.
    pub fn insert_batch_record(
        &mut self,
        record: BatchRecord,
    ) -> Result<(), ArchivalManagerInsertBatchRecordError> {
        // 1 Read batch height from the record.
        let height = record.batch_height;

        // 2 Reject duplicate batch height (append-only).
        if self.in_memory_records.contains_key(&height) {
            return Err(ArchivalManagerInsertBatchRecordError::DuplicateBatchHeight(
                height,
            ));
        }

        // 3 Serialize the batch record for storage.
        let bytes = record
            .serialize()
            .ok_or(ArchivalManagerInsertBatchRecordError::SerializeFailed)?;

        // 4 Insert into the db under the 8-byte height key.
        self.in_db_records
            .insert(height.to_be_bytes(), bytes)
            .map_err(|e| ArchivalManagerInsertBatchRecordError::DbError(e.to_string()))?;

        // 5 Insert the in-memory record keyed by batch height.
        self.in_memory_records.insert(height, record);

        // 6 Return success.
        Ok(())
    }

    /// Returns the full `BatchRecord` for a batch height, if present.
    pub fn batch_record_by_height(&self, batch_height: u64) -> Option<BatchRecord> {
        // 1 Look up the batch record by height.
        self.in_memory_records.get(&batch_height).cloned()
    }

    /// JSON for a full batch record (`BatchRecord::json`), resolved by batch height.
    pub fn batch_record_json_by_height(&self, batch_height: u64) -> Option<Value> {
        // 1 Resolve the batch record from archival history.
        let record = self.batch_record_by_height(batch_height)?;

        // 2 Encode as JSON and return.
        Some(record.json())
    }

    /// Returns the full `BatchRecord` for a batch transaction id, if present.
    pub fn batch_record_by_txid(&self, batch_txid: &[u8; 32]) -> Option<BatchRecord> {
        // 1 Scan in-memory records for the matching batch txid.
        self.in_memory_records
            .values()
            .find(|r| r.batch_txid == *batch_txid)
            .cloned()
    }

    /// Returns the `BatchContainer` whose signed batch txn first input prevout txid matches the request outpoint txid.
    pub fn batch_container_by_prev_payload_outpoint(
        &self,
        prev_payload_outpoint: &OutPoint,
    ) -> Option<BatchContainer> {
        self.in_memory_records
            .values()
            .find_map(|record| {
                record
                    .batch_container
                    .signed_batch_txn
                    .tx_inputs
                    .first()
                    .and_then(|(outpoint, _, _)| {
                        if outpoint.txid == prev_payload_outpoint.txid {
                            Some(record.batch_container.clone())
                        } else {
                            None
                        }
                    })
            })
    }

    /// JSON for a full batch record (`BatchRecord::json`), resolved by batch txid.
    pub fn batch_record_json_by_txid(&self, batch_txid: &[u8; 32]) -> Option<Value> {
        // 1 Resolve the batch record from archival history.
        let record = self.batch_record_by_txid(batch_txid)?;

        // 2 Encode as JSON and return.
        Some(record.json())
    }

    /// Returns an Entry record by entry id.
    pub fn entry_record_by_entry_id(
        &self,
        entry_id: &[u8; 32],
    ) -> Option<(BatchHeight, BatchTxid, BatchTimestamp, EntryId, Entry)> {
        // 1 Walk batches in ascending batch height order.
        for h in sorted_batch_heights(&self.in_memory_records) {
            let record = &self.in_memory_records[&h];
            // 1.1 Walk executed entries within the batch.
            for (stored_id, entry) in &record.entries {
                // 1.1.1 Return on first entry id match.
                if stored_id == entry_id {
                    return Some((
                        record.batch_height,
                        record.batch_txid,
                        record.batch_timestamp,
                        *stored_id,
                        entry.clone(),
                    ));
                }
            }
        }

        // 2 No matching entry id.
        None
    }

    /// Returns a JSON object for an Entry record by entry id.
    pub fn entry_record_json_by_entry_id(&self, entry_id: &[u8; 32]) -> Option<Value> {
        // 1 Resolve the Entry record by entry id.
        let (batch_height, batch_txid, batch_timestamp, resolved_entry_id, entry) =
            self.entry_record_by_entry_id(entry_id)?;

        // 2 Build the JSON object for the Entry record.
        let mut obj = Map::new();
        obj.insert(
            "entry_id".to_string(),
            Value::String(hex::encode(resolved_entry_id)),
        );
        obj.insert(
            "timestamp".to_string(),
            Value::Number(batch_timestamp.into()),
        );
        obj.insert(
            "at_batch_height".to_string(),
            Value::Number(batch_height.into()),
        );
        obj.insert(
            "at_batch_txid".to_string(),
            Value::String(Txid::from_byte_array(batch_txid).to_string()),
        );

        obj.insert("entry".to_string(), entry.json());

        // 3 Return the object.
        Some(Value::Object(obj))
    }

    /// Returns a list of historical Entry records for an account.
    pub fn retrieve_account_history(
        &self,
        account_key: [u8; 32],
    ) -> Vec<(BatchHeight, BatchTxid, BatchTimestamp, EntryId, Entry)> {
        // 1 Initialize the list of historical Entry records.
        let mut historical_entry_records = Vec::new();

        // 2 Walk batches in ascending batch height order.
        for h in sorted_batch_heights(&self.in_memory_records) {
            let record = &self.in_memory_records[&h];
            // 2.1 Walk executed entries within the batch.
            for (stored_entry_id, entry) in &record.entries {
                // 2.1.1 Filter entries that belong to the account.
                if entry_involves_account(entry, account_key) {
                    historical_entry_records.push((
                        record.batch_height,
                        record.batch_txid,
                        record.batch_timestamp,
                        *stored_entry_id,
                        entry.clone(),
                    ));
                }
            }
        }

        // 3 Return the list.
        historical_entry_records
    }

    /// Returns a JSON object for a list of historical Entry records for an account.
    pub fn retrieve_account_history_json(&self, account_key: [u8; 32]) -> Value {
        // 1 Collect historical Entry records for the account.
        let history = self.retrieve_account_history(account_key);

        // 2 Record how many historical Entry records were found.
        let num_entries = history.len();

        // 3 Encode each historical Entry record as a JSON object.
        let entries: Vec<Value> = history
            .into_iter()
            .map(
                |(batch_height, batch_txid, batch_timestamp, entry_id, entry)| {
                    let mut row = Map::new();
                    row.insert("entry_id".to_string(), Value::String(hex::encode(entry_id)));
                    row.insert(
                        "timestamp".to_string(),
                        Value::Number(batch_timestamp.into()),
                    );
                    row.insert(
                        "at_batch_height".to_string(),
                        Value::Number(batch_height.into()),
                    );
                    row.insert(
                        "at_batch_txid".to_string(),
                        Value::String(Txid::from_byte_array(batch_txid).to_string()),
                    );
                    row.insert("entry".to_string(), entry.json());
                    Value::Object(row)
                },
            )
            .collect();

        // 4 Build the JSON object for the list of historical Entry records.
        let mut obj = Map::new();
        obj.insert(
            "num_entries".to_string(),
            Value::Number((num_entries as u64).into()),
        );
        obj.insert("entries".to_string(), Value::Array(entries));

        // 5 Return the object.
        Value::Object(obj)
    }

    /// Returns in-memory `BatchRecord` references sorted by `batch_height`.
    pub fn batch_records(&self) -> Vec<&BatchRecord> {
        sorted_batch_heights(&self.in_memory_records)
            .into_iter()
            .filter_map(|h| self.in_memory_records.get(&h))
            .collect()
    }
}

/// Erases the archival manager database directory for the chain.
pub fn erase_archival_manager(chain: Chain) {
    // 1 Resolve the archival manager db path.
    let path = format!("storage/{}/archival_manager", chain.to_string());

    // 2 Remove the directory tree.
    let _ = std::fs::remove_dir_all(path);
}
