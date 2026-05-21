use crate::constructive::entity::account::account::ext::codec::sbe::decode::error::decode_error::AccountSBEDecodeError;
use crate::constructive::entity::contract::ext::codec::sbe::decode::error::decode_error::ContractSBEDecodeError;

type BytesLength = usize;
type VarbytesByteLength = u16;

/// Errors that can occur when decoding a `CalldataElement` from SBE bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CalldataElementSBEDecodeError {
    CalldataElementSBEEmptyBufferError,
    CalldataElementSBEUnknownTypeTagError(u8),
    CalldataElementSBEBytesTypeMissingLengthIndexError,
    U8(U8SBEDecodeError),
    U16(U16SBEDecodeError),
    U32(U32SBEDecodeError),
    U64(U64SBEDecodeError),
    Bool(BoolSBEDecodeError),
    Account(CallAccountSBEDecodeError),
    Contract(CallContractSBEDecodeError),
    Bytes(BytesSBEDecodeError),
    Varbytes(VarbytesSBEDecodeError),
    Payable(PayableSBEDecodeError),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum U8SBEDecodeError {
    InsufficientPayloadBytes { got: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum U16SBEDecodeError {
    InsufficientPayloadBytes { got: usize },
    BytesConversionError,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum U32SBEDecodeError {
    InsufficientPayloadBytes { got: usize },
    BytesConversionError,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum U64SBEDecodeError {
    InsufficientPayloadBytes { got: usize },
    BytesConversionError,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BoolSBEDecodeError {
    InsufficientPayloadBytes { got: usize },
    InvalidBoolByte(u8),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CallAccountSBEDecodeError {
    AccountSBEDecodeError(AccountSBEDecodeError),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CallContractSBEDecodeError {
    ContractSBEDecodeError(ContractSBEDecodeError),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BytesSBEDecodeError {
    InvalidBytesLength(BytesLength),
    InsufficientPayloadBytes { expected: usize, got: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VarbytesSBEDecodeError {
    InsufficientPayloadBytesForLength { got: usize },
    ByteLengthGreaterThan4095Error(VarbytesByteLength),
    InsufficientPayloadBytesForData { expected: usize, got: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PayableSBEDecodeError {
    InsufficientPayloadBytes { got: usize },
    BytesConversionError,
}

/// Errors that can occur when decoding a calldata element list from SBE bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CalldataElementsSBEDecodeError {
    InsufficientBytesForElementCount { got: usize },
    ElementCountBytesConversionError,
    Element(CalldataElementSBEDecodeError),
    TrailingBytesAfterCalldataList { trailing: usize },
}
