use crate::constructive::bitcoiny::batch_txn::signed_batch_txn::error::construct_error::SignedBatchTxnConstructError;
use crate::constructive::entry::entry::ext::codec::ape::encode::error::encode_error::EntryAPEEncodeError;

/// Errors associated with converting the `ExecCtx` into a `BatchContainer`.
#[derive(Debug, Clone)]
pub enum IntoBatchContainerError {
    BatchInfoNotFoundError,
    EntryAPEEncodeError(EntryAPEEncodeError),
    AggregateBLSSignatureError,
    SignedBatchTxnConstructError(SignedBatchTxnConstructError),
}
