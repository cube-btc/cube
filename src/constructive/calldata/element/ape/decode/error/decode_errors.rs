use crate::constructive::{
    entity::{
        account::ape::decode::error::decode_error::AccountAPEDecodeError,
        contract::ape::decode::error::decode_error::ContractAPEDecodeError,
    },
    valtype::maybe_common::maybe_common::ape::decode::error::decode_error::MaybeCommonAPEDecodeError,
};

/// Type alias for the bytes length.
type BytesLength = usize;

/// Type alias for the varbytes byte length.
type VarbytesByteLength = u16;

/// Enum to represent errors that can occur when decoding a `CallElement` from an Airly Payload Encoding (APE) bit stream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CalldataElementAPEDecodeError {
    U8(U8APEDecodeError),
    U16(U16APEDecodeError),
    U32(U32APEDecodeError),
    U64(U64APEDecodeError),
    Bool(BoolAPEDecodeError),
    Account(CallAccountAPEDecodeError),
    Contract(CallContractAPEDecodeError),
    Bytes(BytesAPEDecodeError),
    Varbytes(VarbytesAPEDecodeError),
    Payable(PayableAPEDecodeError),
}

/// Enum to represent errors that can occur when decoding a `CallElement` from an Airly Payload Encoding (APE) bit stream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum U8APEDecodeError {
    Collect8BitsError,
    ConvertToByteError,
}

/// Enum to represent errors that can occur when decoding a `CallElement` from a bit stream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum U16APEDecodeError {
    Collect16BitsError,
    ConvertToBytesError,
}

/// Enum to represent errors that can occur when decoding a `CallElement` from an Airly Payload Encoding (APE) bit stream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum U32APEDecodeError {
    MaybeCommonShortValAPEDecodingError(MaybeCommonAPEDecodeError),
}

/// Enum to represent errors that can occur when decoding a `CallElement` from an Airly Payload Encoding (APE) bit stream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum U64APEDecodeError {
    MaybeCommonLongValAPEDecodingError(MaybeCommonAPEDecodeError),
}

/// Enum to represent errors that can occur when decoding a `CallElement` from an Airly Payload Encoding (APE) bit stream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BoolAPEDecodeError {
    CollectBoolBitError,
}

/// Enum to represent errors that can occur when decoding a `CallElement` from an Airly Payload Encoding (APE) bit stream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CallAccountAPEDecodeError {
    AccountAPEDecodeError(AccountAPEDecodeError),
}

/// Enum to represent errors that can occur when decoding a `CallElement` from an Airly Payload Encoding (APE) bit stream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CallContractAPEDecodeError {
    ContractAPEDecodeError(ContractAPEDecodeError),
}

/// Enum to represent errors that can occur when decoding a `CallElement` from an Airly Payload Encoding (APE) bit stream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BytesAPEDecodeError {
    InvalidBytesLength(BytesLength),
    CollectDataBitsError,
}

/// Enum to represent errors that can occur when decoding a `CallElement` from an Airly Payload Encoding (APE) bit stream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VarbytesAPEDecodeError {
    CollectVarbytesLengthBitsError,
    ByteLengthGreaterThan4095Error(VarbytesByteLength),
    CollectVarbytesDataBitsError,
}

/// Enum to represent errors that can occur when decoding a `CallElement` from an Airly Payload Encoding (APE) bit stream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PayableAPEDecodeError {
    MaybeCommonShortValAPEDecodingError(MaybeCommonAPEDecodeError),
}
