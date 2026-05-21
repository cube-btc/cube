/// Errors that can occur when decoding an `OpsBudget` from SBE bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OpsBudgetSBEDecodeError {
    /// Not enough bytes for the presence flag, tier, or compact value.
    OpsBudgetSBEInsufficientPayloadBytes { got: usize },
    /// The presence flag is not `0` or `1`.
    OpsBudgetSBEInvalidPresenceFlag { got: u8 },
    /// The tier byte is not in `0`..=`3`.
    OpsBudgetSBEInvalidTierByte { got: u8 },
    /// Compact value bytes could not be parsed as a `u32`.
    OpsBudgetSBEInvalidCompactBytes,
}
