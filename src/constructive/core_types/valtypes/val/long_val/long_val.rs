use crate::constructive::core_types::valtypes::maybe_common::maybe_common::maybe_common::{
    Commonable, MaybeCommonValue, MaybeCommonValueType,
};
use crate::constructive::core_types::valtypes::val::short_val::short_val::ShortVal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LongVal(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LongValTier {
    U8,
    U16,
    U24,
    U32,
    U40,
    U48,
    U56,
    U64,
}

impl LongVal {
    /// Creates a new `LongVal`
    pub fn new(long_val: u64) -> Self {
        Self(long_val)
    }

    /// Returns the core u64 value.
    pub fn value(&self) -> u64 {
        self.0
    }

    /// Determines the tier based on the value range.
    pub fn tier(&self) -> LongValTier {
        match self.value() {
            // 1 byte
            0..=0xFF => LongValTier::U8,
            // 2 bytes
            0x0100..=0xFFFF => LongValTier::U16,
            // 3 bytes
            0x00010000..=0xFFFFFF => LongValTier::U24,
            // 4 bytes
            0x0001000000..=0xFFFFFFFF => LongValTier::U32,
            // 5 bytes
            0x000100000000..=0xFFFFFFFFFF => LongValTier::U40,
            // 6 bytes
            0x00010000000000..=0xFFFFFFFFFFFF => LongValTier::U48,
            // 7 bytes
            0x0001000000000000..=0xFFFFFFFFFFFFFF => LongValTier::U56,
            // 8 bytes
            0x000100000000000000..=u64::MAX => LongValTier::U64,
        }
    }

    /// Returns the compact byte representation of the value.
    pub fn compact_bytes(&self) -> Vec<u8> {
        let value = self.value();
        let bytes = value.to_le_bytes();

        match self.tier() {
            // 1 byte
            LongValTier::U8 => vec![bytes[0]],
            // 2 bytes
            LongValTier::U16 => vec![bytes[0], bytes[1]],
            // 3 bytes
            LongValTier::U24 => vec![bytes[0], bytes[1], bytes[2]],
            // 4 bytes
            LongValTier::U32 => vec![bytes[0], bytes[1], bytes[2], bytes[3]],
            // 5 bytes
            LongValTier::U40 => vec![bytes[0], bytes[1], bytes[2], bytes[3], bytes[4]],
            // 6 bytes
            LongValTier::U48 => vec![bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]],
            // 7 bytes
            LongValTier::U56 => vec![
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
            ],
            // 8 bytes
            LongValTier::U64 => bytes.to_vec(),
        }
    }

    /// Constructs a `LongVal` from its compact byte representation.
    pub fn from_compact_bytes(bytes: &[u8]) -> Option<Self> {
        let value = match bytes.len() {
            // 1 byte
            1 => bytes[0] as u64,
            // 2 bytes
            2 => u16::from_le_bytes([bytes[0], bytes[1]]) as u64,
            // 3 bytes
            3 => {
                let mut buf = [0u8; 8];
                buf[..3].copy_from_slice(bytes);
                u64::from_le_bytes(buf)
            }
            // 4 bytes
            4 => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as u64,
            // 5 bytes
            5 => {
                let mut buf = [0u8; 8];
                buf[..5].copy_from_slice(bytes);
                u64::from_le_bytes(buf)
            }
            // 6 bytes
            6 => {
                let mut buf = [0u8; 8];
                buf[..6].copy_from_slice(bytes);
                u64::from_le_bytes(buf)
            }
            // 7 bytes
            7 => {
                let mut buf = [0u8; 8];
                buf[..7].copy_from_slice(bytes);
                u64::from_le_bytes(buf)
            }
            // 8 bytes
            8 => u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]),
            _ => return None,
        };

        Some(Self(value))
    }
}

/// Implement `Commonable` for `LongVal`.
impl Commonable for LongVal {
    fn maybe_common_value(&self) -> MaybeCommonValue {
        MaybeCommonValue::Long(self.clone())
    }

    fn maybe_common_value_type() -> MaybeCommonValueType {
        MaybeCommonValueType::Long
    }
}

/// Implement `From` for `LongVal` from `ShortVal`.
impl From<ShortVal> for LongVal {
    fn from(val: ShortVal) -> Self {
        LongVal::new(val.value() as u64)
    }
}
