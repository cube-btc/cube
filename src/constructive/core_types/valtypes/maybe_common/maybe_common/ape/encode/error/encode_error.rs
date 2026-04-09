use crate::constructive::entry::entry_fields::valtypes::maybe_common::common::{
    common_long::ape::encode::error::encode_error::CommonLongValAPEEncodeError,
    common_short::ape::encode::error::encode_error::CommonShortValAPEEncodeError,
};

/// Enum to represent errors that can occur when encoding a `MaybeCommon` into a bit vector.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MaybeCommonAPEEncodeError {
    CommonShortValAPEEncodeError(CommonShortValAPEEncodeError),
    CommonLongValAPEEncodeError(CommonLongValAPEEncodeError),
}
