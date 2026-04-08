use crate::constructive::entry::entry_fields::valtypes::maybe_common::maybe_common::maybe_common::{
    Commonable, MaybeCommonValue, MaybeCommonValueType,
};
use crate::constructive::entry::entry_fields::valtypes::val::long_val::long_val::LongVal;
use serde::{Deserialize, Serialize};

/// Represents a short value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShortVal(pub u32);

/// Represents the tier of an uncommon `ShortVal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShortValTier {
    U8,
    U16,
    U24,
    U32,
}

impl ShortVal {
    /// Creates a new `ShortVal`
    pub fn new(short_val: u32) -> Self {
        Self(short_val)
    }

    /// Returns the core u32 value.
    pub fn value(&self) -> u32 {
        self.0
    }

    /// Determines the tier based on the value range.
    pub fn uncommon_tier(&self) -> ShortValTier {
        match self.0 {
            0..=255 => ShortValTier::U8,
            256..=65535 => ShortValTier::U16,
            65536..=16777215 => ShortValTier::U24,
            16777216..=4294967295 => ShortValTier::U32,
        }
    }

    /// Returns the compact byte representation of the value.
    pub fn compact_bytes(&self) -> Vec<u8> {
        let value = self.value();

        match self.uncommon_tier() {
            // 1 byte
            ShortValTier::U8 => vec![value as u8],
            // 2 bytes
            ShortValTier::U16 => {
                let bytes = value.to_le_bytes();
                vec![bytes[0], bytes[1]]
            }
            // 3 bytes
            ShortValTier::U24 => {
                let bytes = value.to_le_bytes();
                vec![bytes[0], bytes[1], bytes[2]]
            }
            // 4 bytes
            ShortValTier::U32 => value.to_le_bytes().to_vec(),
        }
    }

    /// Constructs a `ShortVal` from its compact byte representation.
    pub fn from_compact_bytes(bytes: &[u8]) -> Option<Self> {
        let value = match bytes.len() {
            // 1 byte (u8)
            1 => bytes[0] as u32,
            // 2 bytes (u16)
            2 => u16::from_le_bytes([bytes[0], bytes[1]]) as u32,
            // 3 bytes (u24)
            3 => {
                let mut buf = [0u8; 4];
                buf[..3].copy_from_slice(bytes);
                u32::from_le_bytes(buf)
            }
            // 4 bytes (u32)
            4 => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            _ => return None,
        };

        Some(Self(value))
    }
}

/// Implement `Commonable` for `ShortVal`.
impl Commonable for ShortVal {
    fn maybe_common_value(&self) -> MaybeCommonValue {
        MaybeCommonValue::Short(self.clone())
    }

    fn maybe_common_value_type() -> MaybeCommonValueType {
        MaybeCommonValueType::Short
    }
}

/// Implement `From` for `ShortVal` from `LongVal`.
impl From<LongVal> for ShortVal {
    fn from(val: LongVal) -> Self {
        ShortVal::new(val.value() as u32)
    }
}
