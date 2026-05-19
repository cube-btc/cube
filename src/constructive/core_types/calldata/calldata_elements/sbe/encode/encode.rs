use crate::constructive::calldata::element::calldata_element::CalldataElement;
use crate::constructive::core_types::valtypes::val::long_val::long_val::{LongVal, LongValTier};
use crate::constructive::core_types::valtypes::val::short_val::short_val::{ShortVal, ShortValTier};

type Bytes = Vec<u8>;

fn short_val_tier_byte(tier: ShortValTier) -> u8 {
    match tier {
        ShortValTier::U8 => 0,
        ShortValTier::U16 => 1,
        ShortValTier::U24 => 2,
        ShortValTier::U32 => 3,
    }
}

fn long_val_tier_byte(tier: LongValTier) -> u8 {
    match tier {
        LongValTier::U8 => 0,
        LongValTier::U16 => 1,
        LongValTier::U24 => 2,
        LongValTier::U32 => 3,
        LongValTier::U40 => 4,
        LongValTier::U48 => 5,
        LongValTier::U56 => 6,
        LongValTier::U64 => 7,
    }
}

fn encode_short_val_sbe(short_val: &ShortVal) -> Bytes {
    let mut bytes = Bytes::new();
    bytes.push(short_val_tier_byte(short_val.uncommon_tier()));
    bytes.extend_from_slice(&short_val.compact_bytes());
    bytes
}

fn encode_long_val_sbe(long_val: &LongVal) -> Bytes {
    let mut bytes = Bytes::new();
    bytes.push(long_val_tier_byte(long_val.tier()));
    bytes.extend_from_slice(&long_val.compact_bytes());
    bytes
}

impl CalldataElement {
    /// Structural Byte-scope Encoding (SBE) for a single `CalldataElement`.
    ///
    /// Layout: element-type bytecode (see [`CalldataElementType::bytecode`]), then a
    /// variant-specific payload (fixed-size scalars, tiered compact `ShortVal`/`LongVal`,
    /// length-prefixed `Varbytes`, or nested `Account`/`Contract` SBE).
    pub fn encode_sbe(&self) -> Bytes {
        let mut bytes = self.element_type().bytecode();

        match self {
            CalldataElement::U8(value) => bytes.push(*value),
            CalldataElement::U16(value) => bytes.extend_from_slice(&value.to_le_bytes()),
            CalldataElement::U32(short_val) => bytes.extend(encode_short_val_sbe(short_val)),
            CalldataElement::U64(long_val) => bytes.extend(encode_long_val_sbe(long_val)),
            CalldataElement::Bool(value) => bytes.push(u8::from(*value)),
            CalldataElement::Account(account) => bytes.extend(account.encode_sbe()),
            CalldataElement::Contract(contract) => bytes.extend(contract.encode_sbe()),
            CalldataElement::Bytes(data) => bytes.extend_from_slice(data),
            CalldataElement::Varbytes(data) => {
                let len = data.len() as u16;
                bytes.extend_from_slice(&len.to_le_bytes());
                bytes.extend_from_slice(data);
            }
            CalldataElement::Payable(short_val) => bytes.extend(encode_short_val_sbe(short_val)),
        }

        bytes
    }
}
