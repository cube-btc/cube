use serde::{Deserialize, Serialize};

/// Index of a method on a contract's executable.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MethodIndex {
    pub index: u16,
}

impl MethodIndex {
    pub fn new(index: u16) -> Self {
        Self { index }
    }

    pub fn index(&self) -> u16 {
        self.index
    }
}
