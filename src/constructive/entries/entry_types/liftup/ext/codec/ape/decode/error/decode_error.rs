use crate::constructive::core_types::target::ext::codec::ape::decode::error::decode_error::TargetAPEDecodeError;
use crate::constructive::entity::account::root_account::ext::codec::ape::decode::error::decode_error::RootAccountAPEDecodeError;
use crate::constructive::txo::lift::lift::Lift;
use crate::constructive::valtype::val::short_val::ape::decode::error::decode_error::ShortValAPEDecodeError;
use bitcoin::OutPoint;

/// Airly Payload Encoding (APE) decoding error for `Liftup`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiftupAPEDecodeError {
    RootAccountAPEDecodeError(RootAccountAPEDecodeError),
    TargetAPEDecodeError(TargetAPEDecodeError),
    NumberOfLiftsAPEDecodeError(ShortValAPEDecodeError),
    MissingLiftOutpointError,
    MissingLiftKindBitError,
    UnableToLocateLiftOutpointInUTXOSetError(OutPoint),
    FailedToValidateLiftV1ScriptpubkeyError(Lift),
    FailedToValidateLiftV2ScriptpubkeyError(Lift),
}
