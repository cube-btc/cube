use crate::constructive::core_types::target::ape::encode::error::encode_error::TargetAPEEncodeError;
use crate::constructive::entity::account::root_account::ape::encode::error::encode_error::RootAccountAPEEncodeError;

/// Airly Payload Encoding (APE) encoding error for `Liftup`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiftupAPEEncodeError {
    RootAccountAPEEncodeError(RootAccountAPEEncodeError),
    TargetAPEEncodeError(TargetAPEEncodeError),
}
