use crate::constructive::core_types::valtypes::maybe_common::common::common_long::common_long::CommonLongVal;
use crate::constructive::core_types::valtypes::maybe_common::common::common_short::common_short::CommonShortVal;
use crate::constructive::core_types::valtypes::val::long_val::long_val::LongVal;
use crate::constructive::core_types::valtypes::val::short_val::short_val::ShortVal;
use serde::{Deserialize, Serialize};

/// Trait to determine whether a value is ShortOrLong.
pub trait Commonable {
    fn maybe_common_value(&self) -> MaybeCommonValue;
    /// Associated function to return the type name as a string.
    fn maybe_common_value_type() -> MaybeCommonValueType
    where
        Self: Sized;
}

/// Enum to represent either `ShortVal` or `LongVal`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaybeCommonValueType {
    Short,
    Long,
}

/// Enum to represent either `ShortVal` or `LongVal`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaybeCommonValue {
    Short(ShortVal),
    Long(LongVal),
}

/// Represents a common value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommonVal {
    CommonShort(CommonShortVal),
    CommonLong(CommonLongVal),
}

/// Represents a generic `MaybeCommon` struct that encapsulates either a "common" or an "uncommon" value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaybeCommon<T> {
    Common(CommonVal),
    Uncommon(T),
}

impl<T> MaybeCommon<T>
where
    T: Commonable + Clone + From<ShortVal> + From<LongVal>,
{
    /// Creates a new `MaybeCommon` for `ShortVal` or `LongVal`.
    pub fn new(val: T) -> Self {
        match val.maybe_common_value() {
            MaybeCommonValue::Short(short_val) => {
                // Check if the value is common.
                match CommonShortVal::new(short_val.value()) {
                    Some(common_val) => MaybeCommon::Common(CommonVal::CommonShort(common_val)),
                    None => MaybeCommon::Uncommon(val),
                }
            }
            MaybeCommonValue::Long(long_val) => {
                // Check if the value is within u32 range.
                match long_val.value() <= u32::MAX as u64 {
                    true => {
                        // Check if the value is common.
                        match CommonLongVal::new(long_val.value()) {
                            Some(common_val) => {
                                MaybeCommon::Common(CommonVal::CommonLong(common_val))
                            }
                            None => MaybeCommon::Uncommon(val),
                        }
                    }
                    // If the value is not within u32 range, it is uncommon.
                    false => MaybeCommon::Uncommon(val),
                }
            }
        }
    }

    /// Returns the inner value as either a `ShortVal` or a `LongVal`.
    pub fn value(&self) -> T {
        match self {
            MaybeCommon::Common(val) => match val {
                CommonVal::CommonShort(common_short_val) => {
                    ShortVal::new(common_short_val.value()).into()
                }
                CommonVal::CommonLong(common_long_val) => {
                    LongVal::new(common_long_val.value()).into()
                }
            },
            MaybeCommon::Uncommon(val) => val.to_owned(),
        }
    }

    /// Returns true if the `MaybeCommon` struct is common.
    pub fn is_common(&self) -> bool {
        match self {
            MaybeCommon::Common(_) => true,
            MaybeCommon::Uncommon(_) => false,
        }
    }
}
