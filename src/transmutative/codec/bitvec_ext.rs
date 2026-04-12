use bit_vec::BitVec;

/// A type alias for a vector of bytes.
type Bytes = Vec<u8>;

/// Trait for BitVec extensions.
pub trait BitVecExt {
    /// Converts a BitVec to a payload bytes vector.
    fn to_ape_payload_bytes(&self) -> Bytes;

    /// Converts a payload bytes vector to a BitVec.
    fn from_ape_payload_bytes(bytes: Bytes) -> Option<BitVec>;
}

impl BitVecExt for BitVec {
    fn to_ape_payload_bytes(&self) -> Bytes {
        // 1. Calculate the number of zero-padded bits.
        let bit_length: usize = self.len();

        // 2. Calculate the number of zero-padded bits.
        let bit_length_modulo_8: u8 = (bit_length % 8) as u8;

        // 3. Calculate the number of zero-padded bits.
        let zero_padded_bits_length: u8 = 8 - bit_length_modulo_8;

        // 4. Initizalize the byte vector.
        let mut bytes = Vec::new();

        // 5 Push the zero-padded bits length as the prefix byte.
        bytes.push(zero_padded_bits_length);

        // 6. Extend the byte vector with the bits as bytes.
        bytes.extend_from_slice(&self.to_bytes());

        // 7. Return the byte vector.
        bytes
    }

    fn from_ape_payload_bytes(bytes: Bytes) -> Option<BitVec> {
        // 1. Read the padding-length prefix; missing byte means empty / invalid payload.
        let pad_byte = *bytes.first()?;

        // 2. Reject values that cannot be produced by `to_payload_bytes` (0 is unused; >8 invalid).
        if pad_byte == 0 || pad_byte > 8 {
            return None;
        }

        // 3. `8` encodes “no trailing padding” when bit length is a multiple of 8; else strip `pad_byte` bits.
        let pad_len = if pad_byte == 8 { 0 } else { pad_byte as usize };

        // 4. Decode packed bytes into a BitVec (includes zero padding in the last partial byte).
        let mut bits = BitVec::from_bytes(&bytes[1..]);

        // 5. Drop the zero-padded suffix; underflow means corrupt payload.
        let new_len = bits.len().checked_sub(pad_len)?;
        bits.truncate(new_len);

        // 6. Return the original logical length.
        Some(bits)
    }
}
