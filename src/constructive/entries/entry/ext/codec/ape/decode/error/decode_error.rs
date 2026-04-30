use crate::constructive::entry::entry_kinds::call::ape::decode::error::decode_error::CallEntryAPEDecodeError;
use crate::constructive::entry::entry_kinds::liftup::ext::codec::ape::decode::error::decode_error::LiftupAPEDecodeError;
use crate::constructive::entry::entry_kinds::r#move::ext::codec::ape::decode::error::decode_error::MoveAPEDecodeError;
use crate::constructive::entry::entry_kinds::swapout::ext::codec::ape::decode::error::decode_error::SwapoutAPEDecodeError;

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
    MoveEntryAPEDecodeError(MoveAPEDecodeError),
    CallEntryAPEDecodeError(CallEntryAPEDecodeError),
    LiftupEntryAPEDecodeError(LiftupAPEDecodeError),
    SwapoutEntryAPEDecodeError(SwapoutAPEDecodeError),
}
