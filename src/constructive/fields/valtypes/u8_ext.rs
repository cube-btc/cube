use bit_vec::BitVec;

/// Trait for u8 bitcodec extensions.
pub trait U8Ext {
    /// Convert a u8 to a n-bit BitVec given the bitsize.
    fn to_bits(value: u8, bitsize: u8) -> Option<BitVec>;
    /// Converts bits to a u8 given the bitsize.
    fn from_bits(bits: &BitVec, bitsize: u8) -> Option<u8>;
}

/// Implement `U8Ext` for `u8`.
impl U8Ext for u8 {
    /// Convert a u8 to a n-bit BitVec.
    fn to_bits(value: u8, bitsize: u8) -> Option<BitVec> {
        // Check if the bitsize is valid.
        if bitsize > 8 {
            return None;
        }

        // Convert the u8 to a n-bit BitVec.
        let val_bytes = value.to_le_bytes();

        // Initialize a BitVec.
        let mut val_bits = BitVec::new();

        // Iterate bitsize number of times.
        for i in 0..bitsize {
            val_bits.push((val_bytes[0] >> i) & 1 == 1);
        }

        // Return the BitVec.
        Some(val_bits)
    }

    /// Converts bits to a u8.
    fn from_bits(bits: &BitVec, bitsize: u8) -> Option<u8> {
        // Check if the bitsize is valid.
        if bitsize > 8 {
            return None;
        }

        // Initialize a u8.
        let mut decoded_val = 0u8;

        // Iterate bitsize number of times.
        for i in 0..bitsize {
            let bit = bits[i as usize];
            if bit {
                decoded_val |= 1 << i;
            }
        }

        // Return the u8.
        Some(decoded_val)
    }
}
