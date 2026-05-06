use crate::constructive::core_types::entities::account::root_account::ext::codec::ape::decode::error::decode_error::RootAccountAPEDecodeError;
use crate::constructive::core_types::target::ext::codec::ape::decode::error::decode_error::TargetAPEDecodeError;
use crate::constructive::valtype::val::short_val::ape::decode::error::decode_error::ShortValAPEDecodeError;

/// Airly Payload Encoding (APE) decoding error for `Deploy`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeployAPEDecodeError {
    RootAccountAPEDecodeError(RootAccountAPEDecodeError),
    ProgramLenDecodeError(ShortValAPEDecodeError),
    ProgramBitsCollectError,
    ProgramDecompileError,
    InitialBalanceBitsCollectError,
    InitialBalanceBytesConversionError,
    TargetAPEDecodeError(TargetAPEDecodeError),
}
