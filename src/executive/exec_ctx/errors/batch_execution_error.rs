use crate::constructive::core_types::valtypes::val::long_val::ape::decode::error::decode_error::LongValAPEDecodeError;
use crate::constructive::core_types::valtypes::val::short_val::ape::decode::error::decode_error::ShortValAPEDecodeError;
use crate::constructive::entry::entry::ext::codec::ape::decode::error::decode_error::EntryAPEDecodeError;
use crate::executive::entry_executions::liftup_execution::error::liftup_execution_error::LiftupExecutionError;
use crate::constructive::entries::entry_kinds::liftup::ext::signature::sighash::error::sighash_error::LiftupSighashError;
use bitcoin::OutPoint;

/// A type alias for the current batch sync height tip.
type CurrentBatchSyncHeightTip = u64;

/// A type alias for the new batch height.
type NewBatchHeight = u64;

/// A type alias for the latest payload outpoint.
type LatestPayloadOutpoint = OutPoint;

/// A type alias for the new payload outpoint.
type NewPayloadOutpoint = OutPoint;

/// Errors associated with executing a batch of entries.
#[derive(Debug, Clone)]
pub enum BatchExecutionError {
    InvalidNewBatchHeightError(CurrentBatchSyncHeightTip, NewBatchHeight),
    PayloadTipLocationNotFoundError,
    PayloadOutpointMismatchError(LatestPayloadOutpoint, NewPayloadOutpoint),
    BatchTemplatePayloadBitsConversionError(Vec<u8>),
    DecodePayloadVersionError(ShortValAPEDecodeError),
    DecodeBatchTimestampError(LongValAPEDecodeError),
    DecodeAggregateBLSSignatureError,
    DecodeExtraInCountError(ShortValAPEDecodeError),
    FailedToCollectProjectorPresenceBitError,
    // Iter & collect txin/out errors.
    FailedToIterAndGetPayloadTxInputError,
    FailedToIterAndGetPayloadTxOutputError,
    FailedToIterateExtraInsError,
    FailedToIterAndGetProjectorTxOutputError,
    //
    DecodeEntryError(EntryAPEDecodeError),
    LiftupExecutionError(LiftupExecutionError),
    LiftupSighashError(LiftupSighashError),
    AggregateBLSSignatureVerificationError,
}
