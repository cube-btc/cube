use crate::constructive::core_types::entities::account::root_account::ext::codec::ape::encode::error::encode_error::RootAccountAPEEncodeError;
use crate::constructive::core_types::target::ext::codec::ape::encode::error::encode_error::TargetAPEEncodeError;

/// Airly Payload Encoding (APE) encoding error for `Deploy`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeployAPEEncodeError {
    RootAccountAPEEncodeError(RootAccountAPEEncodeError),
    ProgramCompileError,
    ProgramLenTooLarge(usize),
    TargetAPEEncodeError(TargetAPEEncodeError),
}
