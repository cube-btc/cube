use crate::constructive::valtype::maybe_common::common::common_short::common_short::{
    CommonShortVal, COMMON_SHORT_BITSIZE,
};
use crate::constructive::valtype::maybe_common::common::common_short::ape::decode::error::decode_error::CommonShortValAPEDecodeError;
use crate::constructive::valtype::u8_ext::U8Ext;
use bit_vec::BitVec;

impl CommonShortVal {
    /// Airly Payload Encoding (APE) decoding for `CommonShortVal`.
    ///
    /// This function decodes a common u32 integer from an Airly Payload Encoding (APE) bit stream.
    /// The `CommonShortVal` can be a u32.
    ///
    /// # Arguments
    /// * `bit_stream` - The APE bitstream.
    pub fn decode_ape(
        bit_stream: &mut bit_vec::Iter<'_>,
    ) -> Result<CommonShortVal, CommonShortValAPEDecodeError> {
        // 1 Initialize empty bitvec.
        let mut bits = BitVec::new();

        // 2 Collect 6 bits from the bit stream
        for _ in 0..COMMON_SHORT_BITSIZE {
            let bit = bit_stream
                .next()
                .ok_or(CommonShortValAPEDecodeError::SixBitsCollectError)?;

            // Push the bit to the bitvec.
            bits.push(bit);
        }

        // 3 Decode index.
        let index = u8::from_bits(&bits, COMMON_SHORT_BITSIZE)
            .ok_or(CommonShortValAPEDecodeError::U8ExtFromBitsError)?;

        // 4 Convert the 6-bit index to a CommonInt variant and its u32 value
        let value: u32 = match index {
            0 => 50,
            1 => 100,
            2 => 200,
            3 => 250,
            4 => 500,
            5 => 550,
            6 => 600,
            7 => 750,
            8 => 800,
            9 => 900,
            10 => 999,
            11 => 1_000,
            12 => 1_500,
            13 => 2_000,
            14 => 2_500,
            15 => 3_000,
            16 => 4_000,
            17 => 5_000,
            18 => 5_500,
            19 => 6_000,
            20 => 7_000,
            21 => 7_500,
            22 => 8_000,
            23 => 9_000,
            24 => 9_900,
            25 => 10_000,
            26 => 15_000,
            27 => 20_000,
            28 => 25_000,
            29 => 30_000,
            30 => 35_000,
            31 => 40_000,
            32 => 50_000,
            33 => 55_000,
            34 => 60_000,
            35 => 70_000,
            36 => 75_000,
            37 => 80_000,
            38 => 90_000,
            39 => 99_900,
            40 => 100_000,
            41 => 150_000,
            42 => 200_000,
            43 => 250_000,
            44 => 300_000,
            45 => 350_000,
            46 => 400_000,
            47 => 500_000,
            48 => 550_000,
            49 => 600_000,
            50 => 700_000,
            51 => 750_000,
            52 => 800_000,
            53 => 900_000,
            54 => 1_000_000,
            55 => 2_000_000,
            56 => 2_500_000,
            57 => 5_000_000,
            58 => 5_500_000,
            59 => 10_000_000,
            60 => 25_000_000,
            61 => 50_000_000,
            62 => 75_000_000,
            63 => 100_000_000,

            _ => {
                return Err(CommonShortValAPEDecodeError::UncommonIntegerError);
            }
        };

        // 5 Create a new `CommonVal`.
        let common_short_val = CommonShortVal { value, index };

        // 6 Return the `CommonVal`.
        Ok(common_short_val)
    }
}
