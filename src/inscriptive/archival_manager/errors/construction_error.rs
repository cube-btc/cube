/// Errors associated with constructing the `ArchivalManager`.
#[derive(Debug, Clone)]
pub enum ArchivalConstructionError {
    DBOpenError(sled::Error),
    CorruptRecord(u64),
    UnexpectedDbKeyLength(usize),
    HeightKeyMismatch { db_key_height: u64, record_batch_height: u64 },
}
