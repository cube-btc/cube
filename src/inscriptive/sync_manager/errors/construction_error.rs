/// Errors associated with constructing the `SyncManager`.
#[derive(Debug, Clone)]
pub enum SMConstructionError {
    DBOpenError(sled::Error),
}
