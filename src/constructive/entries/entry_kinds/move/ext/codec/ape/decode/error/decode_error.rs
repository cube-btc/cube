use crate::constructive::core_types::target::ext::codec::ape::decode::error::decode_error::TargetAPEDecodeError;
use crate::constructive::entity::account::account::ext::codec::ape::decode::error::decode_error::AccountAPEDecodeError;
use crate::constructive::entity::account::root_account::ext::codec::ape::decode::error::decode_error::RootAccountAPEDecodeError;
use crate::constructive::valtype::val::short_val::ape::decode::error::decode_error::ShortValAPEDecodeError;

/// Airly Payload Encoding (APE) decoding error for `Move`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveAPEDecodeError {
    RootAccountAPEDecodeError(RootAccountAPEDecodeError),
    AccountAPEDecodeError(AccountAPEDecodeError),
    AmountAPEDecodeError(ShortValAPEDecodeError),
    TargetAPEDecodeError(TargetAPEDecodeError),
}
