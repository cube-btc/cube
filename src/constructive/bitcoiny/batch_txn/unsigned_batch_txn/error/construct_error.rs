#[derive(Debug, Clone)]
pub enum UnsignedBatchTxnConstructError {
    ChangeValueBitcoinTransactionFeeCheckedSubError,
    ChangeValueProjectorValueCheckedSubError,
    ChangeValueSwapoutValueCheckedSubError,
    NewPayloadScriptpubkeyError,
}