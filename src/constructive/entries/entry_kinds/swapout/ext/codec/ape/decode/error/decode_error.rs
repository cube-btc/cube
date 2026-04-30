/// Airly Payload Encoding (APE) decoding error for `Swapout`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SwapoutAPEDecodeError {
    RootAccountAPEDecodeError,
    TargetAPEDecodeError,
    PinlessSelfKindBitCollectError,
    SwapoutTxOutputCollectError,
}
