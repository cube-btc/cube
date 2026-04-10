/// Enum to represent errors that can occur when encoding a `Liftup` to Structural Byte-scope Encoding (SBE) bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LiftupSBEEncodeError {
    // RootAccount SBE blob length prefix.
    /// The `RootAccount` SBE payload is larger than `u32::MAX` and cannot be length-prefixed (`len` is its byte length).
    LiftupSBERootAccountPayloadTooLargeForU32LengthPrefix { len: usize },

    // lift_prevtxos count prefix.
    /// The number of `Lift` prevtxos is larger than `u32::MAX` and cannot be count-prefixed (`count` is `lift_prevtxos.len()`).
    LiftupSBETooManyLiftsForU32CountPrefix { count: usize },
}
