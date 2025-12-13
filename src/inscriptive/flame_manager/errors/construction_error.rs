/// Errors associated with constructing the `FlameManager`.
#[derive(Debug, Clone)]
pub enum FMConstructionError {
    /// The database open error.
    DBOpenError(sled::Error),
}
