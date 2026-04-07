/// Sync inflight type.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SyncMode {
    InFlight,
    ConfirmedOnly,
}

impl ToString for SyncMode {
    fn to_string(&self) -> String {
        match self {
            SyncMode::InFlight => "in-flight".to_string(),
            SyncMode::ConfirmedOnly => "confirmed-only".to_string(),
        }
    }
}
