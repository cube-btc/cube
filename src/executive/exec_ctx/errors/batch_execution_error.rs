use crate::constructive::core_types::valtypes::val::long_val::ape::decode::error::decode_error::LongValAPEDecodeError;
use crate::constructive::core_types::valtypes::val::short_val::ape::decode::error::decode_error::ShortValAPEDecodeError;
use crate::constructive::entry::entry::ext::codec::ape::decode::error::decode_error::EntryAPEDecodeError;
use crate::constructive::entries::entry_kinds::liftup::ext::signature::sighash::error::sighash_error::LiftupSighashError;
use crate::executive::entry_executions::liftup_execution::error::liftup_execution_error::LiftupExecutionError;
use crate::executive::exec_ctx::errors::apply_changes_error::ApplyChangesError;
use bitcoin::OutPoint;

/// A type alias for the current batch sync height tip.
type CurrentBatchSyncHeightTip = u64;

/// A type alias for the new batch height.
type NewBatchHeight = u64;

/// A type alias for the prev payload outpoint from the batch container.
type PrevPayloadOutpointFromBatchContainer = OutPoint;

/// A type alias for the payload tip outpoint.
type PayloadTipOutpoint = OutPoint;

/// Errors associated with executing a batch of entries.
#[derive(Debug, Clone)]
pub enum BatchExecutionError {
    InvalidNewBatchHeightError(CurrentBatchSyncHeightTip, NewBatchHeight),
    BatchTemplatePayloadBitsConversionError(Vec<u8>),
    DecodePayloadVersionError(ShortValAPEDecodeError),
    DecodeBatchTimestampError(LongValAPEDecodeError),
    DecodeAggregateBLSSignatureError,
    DecodeExpiredProjectorsCountError(ShortValAPEDecodeError),
    FailedToCollectProjectorPresenceBitError,
    // Iter & collect txin/out errors.
    FailedToIterAndGetPayloadTxInputError,
    PayloadTipLocationNotFoundError,
    PayloadOutpointMismatchError(PrevPayloadOutpointFromBatchContainer, PayloadTipOutpoint),
    FailedToIterAndGetPayloadTxOutputError,
    FailedToIterateExpiredProjectorsError,
    FailedToIterAndGetProjectorTxOutputError,
    //
    DecodeEntryError(EntryAPEDecodeError),
    LiftupExecutionError(LiftupExecutionError),
    LiftupSighashError(LiftupSighashError),
    AggregateBLSSignatureVerificationError,
    ExecutedEntryIdError,
    ApplyChangesError(ApplyChangesError),
}
