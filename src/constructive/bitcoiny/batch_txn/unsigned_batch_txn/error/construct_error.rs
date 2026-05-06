#[derive(Debug, Clone)]
pub enum UnsignedBatchTxnConstructError {
    ChangeValueBitcoinTransactionFeeCheckedSubError,
    BitcoinTransactionFeeFromFeerateCheckedMulError,
    ChangeValueProjectorValueCheckedSubError,
    ChangeValueSwapoutValueCheckedSubError,
    NewPayloadScriptpubkeyError,
}