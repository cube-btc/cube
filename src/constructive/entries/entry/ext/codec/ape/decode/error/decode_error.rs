use crate::constructive::entry::entry_types::call::ape::decode::error::decode_error::CallEntryAPEDecodeError;
use crate::constructive::entry::entry_types::liftup::ext::codec::ape::decode::error::decode_error::LiftupAPEDecodeError;

/// Enum to represent errors that can occur when decoding an `Entry` from an Airly Payload Encoding (APE) bitstream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryAPEDecodeError {
    CommonUncommonBranchBitCollectError,
    MoveOrCallBitCollectError,
    LiquidityOrOuterBranchBitCollectError,
    AddOrSubBitCollectError,
    GatewayOrOuterRightBranchBitCollectError,
    LiftupOrSwapoutBitCollectError,
    OuterLowermostOrReservedBranchBitCollectError,
    DeployOrConfigBitCollectError,
    ReservedBranchEncounteredError,
    //
    CallEntryAPEDecodeError(CallEntryAPEDecodeError),
    LiftupEntryAPEDecodeError(LiftupAPEDecodeError),
}
