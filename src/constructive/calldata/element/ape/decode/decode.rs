use crate::constructive::calldata::element::ape::decode::error::decode_errors::{
    BoolAPEDecodeError, BytesAPEDecodeError, CallAPEDecodeError, CallAccountAPEDecodeError,
    CallContractAPEDecodeError, PayableAPEDecodeError, U16APEDecodeError, U32APEDecodeError,
    U64APEDecodeError, U8APEDecodeError, VarbytesAPEDecodeError,
};
use crate::constructive::calldata::element::element::CallElement;
use crate::constructive::calldata::element_type::CallElementType;
use crate::constructive::entity::account::account::Account;
use crate::constructive::entity::contract::contract::Contract;
use crate::constructive::valtype::maybe_common::maybe_common::maybe_common::MaybeCommon;
use crate::constructive::valtype::val::long_val::long_val::LongVal;
use crate::constructive::valtype::val::short_val::short_val::ShortVal;
use crate::inscriptive::registery_manager::registery_manager::REGISTERY_MANAGER;

use bit_vec::BitVec;

impl CallElement {
    /// Decodes a `CallElement` from an Airly Payload Encoding (APE) bit stream.
    ///
    /// This function decodes a `CallElement` from an Airly Payload Encoding (APE) bit stream.
    /// The `CallElement` can be a u8, u16, u32, u64, bool, account, contract, bytes, varbytes, or payable.
    ///
    /// # Arguments
    /// * `bit_stream` - The APE bitstream.
    /// * `element_type` - The type of the `CallElement`.
    /// * `registery_manager` - The `Registery Manager`.
    /// * `decode_rank_as_longval` - Whether to decode the rank value as a `LongVal` or a `ShortVal`.
    pub async fn decode_ape<'a>(
        bit_stream: &mut bit_vec::Iter<'_>,
        element_type: CallElementType,
        registery_manager: &REGISTERY_MANAGER,
        decode_rank_as_longval: bool,
    ) -> Result<Self, CallAPEDecodeError> {
        // Match on the calldata element type.
        match element_type {
            // Decode the u8.
            CallElementType::U8 => {
                // Create a new bit vector.
                let mut bits = BitVec::new();

                // Collect 8 bits.
                for _ in 0..8 {
                    bits.push(
                        bit_stream
                            .next()
                            .ok_or(CallAPEDecodeError::U8(U8APEDecodeError::Collect8BitsError))?,
                    );
                }

                // Convert to byte.
                let byte: [u8; 1] = bits
                    .to_bytes()
                    .try_into()
                    .map_err(|_| CallAPEDecodeError::U8(U8APEDecodeError::ConvertToByteError))?;

                // Convert byte to a u8.
                let value = byte[0];

                // Construct the `CalldataElement`.
                let element = CallElement::U8(value);

                // Return the element.
                Ok(element)
            }

            // Decode the u16.
            CallElementType::U16 => {
                // Create a new bit vector.
                let mut bits = BitVec::new();

                // Collect 16 bits.
                for _ in 0..16 {
                    bits.push(bit_stream.next().ok_or(CallAPEDecodeError::U16(
                        U16APEDecodeError::Collect16BitsError,
                    ))?);
                }

                // Convert to bytes.
                let bytes: [u8; 2] = bits
                    .to_bytes()
                    .try_into()
                    .map_err(|_| CallAPEDecodeError::U16(U16APEDecodeError::ConvertToBytesError))?;

                // Convert the bytes to a u16.
                let value = u16::from_le_bytes(bytes);

                // Construct the `CalldataElement`.
                let element = CallElement::U16(value);

                // Return the element.
                Ok(element)
            }

            // Decode the u32.
            CallElementType::U32 => {
                // Decode the `ShortVal` from `MaybeCommon<ShortVal>`.
                let short_val = MaybeCommon::<ShortVal>::decode_ape(bit_stream)
                    .map_err(|e| {
                        CallAPEDecodeError::U32(
                            U32APEDecodeError::MaybeCommonShortValAPEDecodingError(e),
                        )
                    })?
                    .value();

                // Construct the `CalldataElement`.
                let element = CallElement::U32(short_val);

                // Return the element.
                Ok(element)
            }

            // Decode the u64.
            CallElementType::U64 => {
                // Decode the `LongVal` from `MaybeCommon<LongVal>`.
                let long_val = MaybeCommon::<LongVal>::decode_ape(bit_stream)
                    .map_err(|e| {
                        CallAPEDecodeError::U64(
                            U64APEDecodeError::MaybeCommonLongValAPEDecodingError(e),
                        )
                    })?
                    .value();

                let element = CallElement::U64(long_val);

                // Return the element.
                Ok(element)
            }

            // Decode the bool.
            CallElementType::Bool => {
                // Collect the bool value by iterating over a single bit.
                let bool = bit_stream.next().ok_or(CallAPEDecodeError::Bool(
                    BoolAPEDecodeError::CollectBoolBitError,
                ))?;

                // Construct the `CalldataElement`.
                let element = CallElement::Bool(bool);

                // Return the element.
                Ok(element)
            }

            // Decode the `Account`.
            CallElementType::Account => {
                // Decode the `Account`.
                let account =
                    Account::decode_ape(bit_stream, &registery_manager, decode_rank_as_longval)
                        .await
                        .map_err(|e| {
                            CallAPEDecodeError::Account(
                                CallAccountAPEDecodeError::AccountAPEDecodeError(e),
                            )
                        })?;

                // Construct the `CalldataElement`.
                let element = CallElement::Account(account);

                // Return the element.
                Ok(element)
            }

            // Decode the `Contract`.
            CallElementType::Contract => {
                // Decode the `Contract`.
                let contract =
                    Contract::decode_ape(bit_stream, &registery_manager, decode_rank_as_longval)
                        .await
                        .map_err(|e| {
                            CallAPEDecodeError::Contract(
                                CallContractAPEDecodeError::ContractAPEDecodeError(e),
                            )
                        })?;

                // Construct the `CallElement`.
                let element = CallElement::Contract(contract);

                // Return the element.
                Ok(element)
            }

            // Decode the `Bytes1-256`.
            CallElementType::Bytes(index) => {
                // Byte length is the index + 1.
                let byte_length = index as usize + 1;

                // Check if the data length is valid.
                if byte_length < 1 || byte_length > 256 {
                    return Err(CallAPEDecodeError::Bytes(
                        BytesAPEDecodeError::InvalidBytesLength(byte_length),
                    ));
                }

                // Get the number of bits to collect.
                let bit_length = byte_length as usize * 8;

                // Collect the data bits.
                let mut data_bits = BitVec::new();
                for _ in 0..bit_length {
                    data_bits.push(bit_stream.next().ok_or(CallAPEDecodeError::Bytes(
                        BytesAPEDecodeError::CollectDataBitsError,
                    ))?);
                }

                // Convert the bits to data bytes.
                let data_bytes = data_bits.to_bytes();

                // Construct the `CalldataElement`.
                let element = CallElement::Bytes(data_bytes);

                // Return the element.
                Ok(element)
            }

            // Decode the `Varbytes`.
            CallElementType::Varbytes => {
                // Initialize a bit vector to fill with byte length.
                let mut byte_length_bits = BitVec::new();

                // Collect 12 bits representing the byte length.
                // Supported byte-length range: 0 to 4095.
                for _ in 0..12 {
                    byte_length_bits.push(bit_stream.next().ok_or(
                        CallAPEDecodeError::Varbytes(
                            VarbytesAPEDecodeError::CollectVarbytesLengthBitsError,
                        ),
                    )?);
                }

                // Convert the byte length bits to a u16.
                let byte_length = convert_12_bits_to_u16(&byte_length_bits);

                // Return an error if the byte length is greater than 4095.
                if byte_length > 4095 {
                    return Err(CallAPEDecodeError::Varbytes(
                        VarbytesAPEDecodeError::ByteLengthGreaterThan4095Error(byte_length),
                    ));
                }

                // If the data length is 0, return an empty `Varbytes`.
                if byte_length == 0 {
                    return Ok(CallElement::Varbytes(vec![]));
                }

                // Convert to bit length.
                let bit_length = byte_length as usize * 8;

                // Initialize bit vector to fill with data.
                let mut data_bits = BitVec::new();

                // Collect the data bit by bit.
                for _ in 0..bit_length {
                    data_bits.push(bit_stream.next().ok_or(CallAPEDecodeError::Varbytes(
                        VarbytesAPEDecodeError::CollectVarbytesDataBitsError,
                    ))?);
                }

                // Convert the bits to bytes.
                let data_bytes = data_bits.to_bytes();

                // Construct `CalldataElement` from the bytes.
                let element = CallElement::Varbytes(data_bytes);

                // Return the element.
                Ok(element)
            }

            // Decode the `Payable`.
            CallElementType::Payable => {
                // Decode the `ShortVal` from `MaybeCommon<ShortVal>`.
                let short_val = MaybeCommon::<ShortVal>::decode_ape(bit_stream)
                    .map_err(|e| {
                        CallAPEDecodeError::Payable(
                            PayableAPEDecodeError::MaybeCommonShortValAPEDecodingError(e),
                        )
                    })?
                    .value();

                // Construct the `CalldataElement`.
                let element = CallElement::Payable(short_val);

                // Return the element.
                Ok(element)
            }
        }
    }
}

/// Converts 12 bits to a u16.
fn convert_12_bits_to_u16(bits: &BitVec) -> u16 {
    // Initialize a u16 value.
    let mut byte_length = 0u16;

    // Iterate over 12 bits.
    for i in 0..12 {
        let bit = bits[i];
        if bit {
            byte_length |= 1 << i;
        }
    }

    // Return the u16 value.
    byte_length
}
