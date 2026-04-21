use crate::constructive::core_types::target::ext::codec::ape::encode::error::encode_error::TargetAPEEncodeError;
use crate::constructive::entity::account::account::ext::codec::ape::encode::error::encode_error::AccountAPEEncodeError;
use crate::constructive::entity::account::root_account::ext::codec::ape::encode::error::encode_error::RootAccountAPEEncodeError;

/// Airly Payload Encoding (APE) encoding error for `Move`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MoveAPEEncodeError {
    RootAccountAPEEncodeError(RootAccountAPEEncodeError),
    AccountAPEEncodeError(AccountAPEEncodeError),
    TargetAPEEncodeError(TargetAPEEncodeError),
}
