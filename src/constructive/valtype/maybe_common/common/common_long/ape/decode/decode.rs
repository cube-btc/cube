use crate::constructive::valtype::maybe_common::common::common_long::common_long::{
    CommonLongVal, COMMON_LONG_BITSIZE,
};
use crate::constructive::valtype::maybe_common::common::common_long::ape::decode::error::decode_error::CommonLongValAPEDecodeError;
use crate::constructive::valtype::u8_ext::U8Ext;
use bit_vec::BitVec;

impl CommonLongVal {
    /// Airly Payload Encoding (APE) decoding for `CommonLongVal`.
    ///
    /// This function decodes a common u64 integer from an Airly Payload Encoding (APE) bit stream.
    /// The `CommonLongVal` can be a u64.
    ///
    /// # Arguments
    /// * `bit_stream` - The APE bitstream.
    pub fn decode_ape(
        bit_stream: &mut bit_vec::Iter<'_>,
    ) -> Result<CommonLongVal, CommonLongValAPEDecodeError> {
        // 1 Initialize empty bitvec.
        let mut bits = BitVec::new();

        // 2 Collect 7 bits from the bit stream
        for _ in 0..COMMON_LONG_BITSIZE {
            let bit = bit_stream
                .next()
                .ok_or(CommonLongValAPEDecodeError::SevenBitsCollectError)?;

            // Push the bit to the bitvec.
            bits.push(bit);
        }

        // 3 Decode index.
        let index = u8::from_bits(&bits, COMMON_LONG_BITSIZE)
            .ok_or(CommonLongValAPEDecodeError::U8ExtFromBitsError)?;

        // 4 Convert the 6-bit index to a CommonInt variant and its u32 value
        let value: u64 = match index {
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
            64 => 150_000_000,
            65 => 200_000_000,
            66 => 250_000_000,
            67 => 300_000_000,
            68 => 350_000_000,
            69 => 400_000_000,
            70 => 500_000_000,
            71 => 550_000_000,
            72 => 600_000_000,
            73 => 700_000_000,
            74 => 750_000_000,
            75 => 800_000_000,
            76 => 900_000_000,
            77 => 1_000_000_000,
            78 => 2_000_000_000,
            79 => 2_500_000_000,
            80 => 5_000_000_000,
            81 => 5_500_000_000,
            82 => 10_000_000_000,
            83 => 25_000_000_000,
            84 => 50_000_000_000,
            85 => 75_000_000_000,
            86 => 100_000_000_000,
            87 => 150_000_000_000,
            88 => 200_000_000_000,
            89 => 250_000_000_000,
            90 => 300_000_000_000,
            91 => 350_000_000_000,
            92 => 400_000_000_000,
            93 => 500_000_000_000,
            94 => 550_000_000_000,
            95 => 600_000_000_000,
            96 => 700_000_000_000,
            97 => 750_000_000_000,
            98 => 800_000_000_000,
            99 => 900_000_000_000,
            100 => 1_000_000_000_000,
            101 => 2_000_000_000_000,
            102 => 2_500_000_000_000,
            103 => 5_000_000_000_000,
            104 => 5_500_000_000_000,
            105 => 10_000_000_000_000,
            106 => 25_000_000_000_000,
            107 => 50_000_000_000_000,
            108 => 75_000_000_000_000,
            109 => 100_000_000_000_000,
            110 => 150_000_000_000_000,
            111 => 200_000_000_000_000,
            112 => 250_000_000_000_000,
            113 => 300_000_000_000_000,
            114 => 350_000_000_000_000,
            115 => 400_000_000_000_000,
            116 => 500_000_000_000_000,
            117 => 550_000_000_000_000,
            118 => 600_000_000_000_000,
            119 => 700_000_000_000_000,
            120 => 750_000_000_000_000,
            121 => 800_000_000_000_000,
            122 => 900_000_000_000_000,
            123 => 1_000_000_000_000_000,
            124 => 2_000_000_000_000_000,
            125 => 2_500_000_000_000_000,
            126 => 5_000_000_000_000_000,
            127 => 10_000_000_000_000_000,
            _ => {
                return Err(CommonLongValAPEDecodeError::UncommonIntegerError);
            }
        };

        // 5 Create a new `CommonLongVal`.
        let common_long_val = CommonLongVal { value, index };

        // 6 Return the `CommonLongVal`.
        Ok(common_long_val)
    }
}
