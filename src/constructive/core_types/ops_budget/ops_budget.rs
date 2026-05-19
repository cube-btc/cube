use serde::{Deserialize, Serialize};

/// Optional execution ops budget carried on an entry.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OpsBudget {
    pub ops_budget: Option<u32>,
}

impl OpsBudget {
    /// Creates a new `OpsBudget`.
    pub fn new(ops_budget: Option<u32>) -> OpsBudget {
        OpsBudget { ops_budget }
    }
}
