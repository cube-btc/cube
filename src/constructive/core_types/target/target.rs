use serde::{Deserialize, Serialize};

/// The `Target` struct represents when a particular `Entry` has been targeted at its creation time.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Target {
    // The batch height at which the Entry has been targeted.
    pub targeted_at_batch_height: u64,
}

impl Target {
    /// Creates a new `Target` struct.
    pub fn new(targeted_at_batch_height: u64) -> Target {
        Target {
            targeted_at_batch_height,
        }
    }
}
