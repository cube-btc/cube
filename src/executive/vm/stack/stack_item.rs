use std::fmt;

/// The type of items in the stack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackItem(Vec<u8>);

impl StackItem {
    /// Creates a new stack item.
    pub fn new(item: Vec<u8>) -> Self {
        Self(item)
    }

    /// Returns a true (0x01) stack item.
    pub fn true_item() -> Self {
        Self(vec![0x01])
    }

    /// Returns a false (empty) stack item.
    pub fn false_item() -> Self {
        Self(vec![])
    }

    /// Returns the bytes of the stack item.
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    /// Returns the length of the stack item.   
    pub fn len(&self) -> u32 {
        self.0.len() as u32
    }

    /// Returns whether the stack item is false.
    pub fn is_false(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns whether the stack item is true.
    pub fn is_true(&self) -> bool {
        !self.is_false()
    }
}

impl fmt::Display for StackItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}
