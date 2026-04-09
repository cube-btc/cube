use crate::constructive::entity::account::root_account::ape::decode::error::decode_error::RootAccountAPEDecodeError;
use crate::constructive::valtype::val::short_val::ape::decode::error::decode_error::ShortValAPEDecodeError;
use bitcoin::OutPoint;

/// Airly Payload Encoding (APE) decoding error for `Liftup`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiftupAPEDecodeError {
    RootAccountAPEDecodeError(RootAccountAPEDecodeError),
    NumberOfLiftsAPEDecodeError(ShortValAPEDecodeError),
    MissingLiftOutpointError,
    MissingLiftKindBitError,
    UnableToLocateLiftOutpointInUTXOSetError(OutPoint),
}
