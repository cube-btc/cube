/// Enum to represent errors that can occur when decoding a `Swapout` from Structural Byte-scope Encoding (SBE) bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SwapoutSBEDecodeError {
    InvalidEntryKindByteError { expected: u8, got: u8 },
    SwapoutSBEInsufficientBytesForRootAccountLengthPrefix { got_total: usize },
    SwapoutSBERootAccountLengthPrefixBytesConversionError,
    SwapoutSBERootAccountLengthPrefixExceedsPayload { root_len: usize, got_after_prefix: usize },
    SwapoutSBERootAccountDecodeError,
    SwapoutSBEInsufficientBytesForTargetAndAmount { got_total: usize },
    SwapoutSBEInsufficientBytesForPinlessSelfKind { got_total: usize },
    SwapoutSBEInvalidPinlessSelfKindByte(u8),
    SwapoutSBETrailingBytesAfterSwapout { trailing: usize },
}
