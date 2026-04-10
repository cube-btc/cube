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

    /// Validates the `Target` against the execution height.
    pub fn validate(&self, execution_height: u64) -> Result<(), (u64, u64)> {
        // 1 Compute the gap between the execution height and the targeted height.
        let gap = execution_height
            .checked_sub(self.targeted_at_batch_height)
            .ok_or((self.targeted_at_batch_height, execution_height))?;

        // 2 Check if the gap is greater than 4.
        if gap > 4 {
            return Err((self.targeted_at_batch_height, execution_height));
        }

        // 3 Return Ok.
        Ok(())
    }
}
