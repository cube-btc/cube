/// Errors associated with inserting a `BatchRecord` into archival storage.
#[derive(Debug, Clone)]
pub enum ArchivalManagerInsertBatchRecordError {
    DuplicateBatchHeight(u64),
    SerializeFailed,
    DbError(String),
}
