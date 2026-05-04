/// Errors that can occur when encoding a `Config` to Structural Byte-scope Encoding (SBE) bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConfigSBEEncodeError {
    ConfigSBERootAccountPayloadTooLargeForU32LengthPrefix { len: usize },
    ConfigSBESecondaryAggregationKeyPayloadTooLargeForU32LengthPrefix { len: usize },
    ConfigSBEFlameConfigPayloadTooLargeForU32LengthPrefix { len: usize },
}
