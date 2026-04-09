use serde::{Deserialize, Serialize};

/// The inner value `AtomicVal` represents.
type Value = u8;

/// The upper bound of the `AtomicVal`.
type UpperBound = u8;

/// Atomic compact value representation from 0 to 255.
///
/// Used for very small value representations such as contract method call indexes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AtomicVal(Value, UpperBound);

impl AtomicVal {
    /// Creates a new `AtomicVal`
    pub fn new(value: u8, upper_bound: UpperBound) -> Self {
        Self(value, upper_bound)
    }

    /// Returns the core u8 value.
    pub fn value(&self) -> u8 {
        self.0
    }

    /// Returns the upper bound.
    pub fn upper_bound(&self) -> UpperBound {
        self.1
    }

    /// Returns the bitsize tier of `AtomicVal`.
    pub fn bitsize(upper_bound: UpperBound) -> u8 {
        match upper_bound {
            0..=1 => 1,
            2..=3 => 2,
            4..=7 => 3,
            8..=15 => 4,
            16..=31 => 5,
            32..=63 => 6,
            64..=127 => 7,
            128..=255 => 8,
        }
    }
}
