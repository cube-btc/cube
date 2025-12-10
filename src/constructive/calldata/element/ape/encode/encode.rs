use crate::constructive::calldata::element::ape::encode::error::encode_error::CalldataElementAPEEncodeError;
use crate::constructive::calldata::element::element::CalldataElement;
use bit_vec::BitVec;

impl CalldataElement {
    /// Encodes the `CallElement` as an Airly Payload Encoding (APE) bit vector.
    ///
    /// This function encodes the `CallElement` as an Airly Payload Encoding (APE) bit vector.
    /// The `CallElement` can be a u8, u16, u32, u64, bool, account, contract, bytes, varbytes, or payable.
    ///
    /// # Arguments
    /// * `&self` - The `CallElement` to encode.
    /// * `encode_rank_as_longval` - Whether to encode the rank value as a `LongVal` or a `ShortVal`.
    ///
    /// # Returns
    pub fn encode_ape(
        &self,
        encode_account_rank_as_longval: bool,
        encode_contract_rank_as_longval: bool,
    ) -> Result<BitVec, CalldataElementAPEEncodeError> {
        // Match on the element type.
        match self {
            CalldataElement::U8(u8_value) => {
                // Get the u8 value.
                let value = *u8_value;

                // Convert value to bytes.
                let byte: [u8; 1] = value.to_le_bytes();

                // Convert bytes to bits.
                let bits = BitVec::from_bytes(&byte);

                // Return the bits.
                Ok(bits)
            }
            CalldataElement::U16(u16_value) => {
                // Get the u16 value.
                let value = *u16_value;

                // Convert value to bytes.
                let bytes: [u8; 2] = value.to_le_bytes();

                // Convert bytes to bits.
                let bits = BitVec::from_bytes(&bytes);

                // Return the bits.
                Ok(bits)
            }
            CalldataElement::U32(short_val) => {
                // Encode the `ShortVal`.
                let bits = short_val.encode_ape();

                // Return the bits.
                Ok(bits)
            }
            CalldataElement::U64(long_val) => {
                // Encode the `LongVal`.
                let bits = long_val.encode_ape();

                // Return the bits.
                Ok(bits)
            }
            CalldataElement::Bool(value) => {
                // Get the bool value.
                let bool = *value;

                // Initialize bit vector.
                let mut bits = BitVec::new();

                // Push the bool value.
                bits.push(bool);

                // Return the bits.
                Ok(bits)
            }
            CalldataElement::Account(account) => {
                // Encode the `Account`.
                let bits = account
                    .encode_ape(encode_account_rank_as_longval)
                    .map_err(|e| CalldataElementAPEEncodeError::AccountAPEEncodeError(e))?;

                // Return the bits.
                Ok(bits)
            }
            CalldataElement::Contract(contract) => {
                // Encode the `Contract`.
                let bits = contract
                    .encode_ape(encode_contract_rank_as_longval)
                    .map_err(|e| CalldataElementAPEEncodeError::ContractAPEEncodeError(e))?;

                // Return the bits.
                Ok(bits)
            }
            CalldataElement::Bytes(bytes) => {
                // Encode the bytes.
                let bits = BitVec::from_bytes(bytes);

                // Return the bits.
                Ok(bits)
            }
            CalldataElement::Varbytes(bytes) => {
                // Initialize bit vector to fill with length plus data.
                let mut bits = BitVec::new();

                // Get the byte length value.
                let byte_length = bytes.len() as u16;

                // Byte length as 2 bytes.
                let byte_length_bits = convert_u16_to_12_bits(byte_length);

                // Extend the bit vector with the byte length.
                bits.extend(byte_length_bits);

                // If data length is 0, return the bit vector with length-bits-only.
                // This is to avoid encoding empty data, as data can be empty.
                if byte_length == 0 {
                    return Ok(bits);
                }

                // Get the data bits.
                let data_bits = BitVec::from_bytes(bytes);

                // Extend the bit vector with the data bits.
                bits.extend(data_bits);

                // Return the bits.
                Ok(bits)
            }
            CalldataElement::Payable(short_val) => {
                // Encode the `ShortVal`.
                let bits = short_val.encode_ape();

                // Return the bits.
                Ok(bits)
            }
        }
    }
}

/// Converts a u16 to 12 bits.
fn convert_u16_to_12_bits(value: u16) -> BitVec {
    // Byte length as 2 bytes.
    let byte_length_bytes = value.to_le_bytes();

    // Initialize byte length bits.
    let mut byte_length_bits = BitVec::new();

    // Convert byte length to bits.
    for i in 0..12 {
        let byte_idx = i / 8;
        let bit_idx = i % 8;
        byte_length_bits.push((byte_length_bytes[byte_idx] >> bit_idx) & 1 == 1);
    }

    // Return the bits.
    byte_length_bits
}
