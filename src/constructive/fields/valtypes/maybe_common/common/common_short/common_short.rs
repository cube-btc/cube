use serde::{Deserialize, Serialize};

/// The bitsize of the `CommonShortVal`.
pub const COMMON_SHORT_BITSIZE: u8 = 6;

/// `CommonShortVal` represents a common u32 integer value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommonShortVal {
    pub value: u32,
    pub index: u8,
}

impl CommonShortVal {
    /// Creates a new `CommonShortVal` from a `u32` integer.
    pub fn new(value: u32) -> Option<CommonShortVal> {
        // Get the index.
        let index = match value {
            50 => 0,
            100 => 1,
            200 => 2,
            250 => 3,
            500 => 4,
            550 => 5,
            600 => 6,
            750 => 7,
            800 => 8,
            900 => 9,
            999 => 10,
            1_000 => 11,
            1_500 => 12,
            2_000 => 13,
            2_500 => 14,
            3_000 => 15,
            4_000 => 16,
            5_000 => 17,
            5_500 => 18,
            6_000 => 19,
            7_000 => 20,
            7_500 => 21,
            8_000 => 22,
            9_000 => 23,
            9_900 => 24,
            10_000 => 25,
            15_000 => 26,
            20_000 => 27,
            25_000 => 28,
            30_000 => 29,
            35_000 => 30,
            40_000 => 31,
            50_000 => 32,
            55_000 => 33,
            60_000 => 34,
            70_000 => 35,
            75_000 => 36,
            80_000 => 37,
            90_000 => 38,
            99_900 => 39,
            100_000 => 40,
            150_000 => 41,
            200_000 => 42,
            250_000 => 43,
            300_000 => 44,
            350_000 => 45,
            400_000 => 46,
            500_000 => 47,
            550_000 => 48,
            600_000 => 49,
            700_000 => 50,
            750_000 => 51,
            800_000 => 52,
            900_000 => 53,
            1_000_000 => 54,
            2_000_000 => 55,
            2_500_000 => 56,
            5_000_000 => 57,
            5_500_000 => 58,
            10_000_000 => 59,
            25_000_000 => 60,
            50_000_000 => 61,
            75_000_000 => 62,
            100_000_000 => 63,

            _ => return None, // Invalid value
        };

        // Create a new `CommonShortVal`.
        let common_short_val = CommonShortVal { value, index };

        // Return the `CommonShortVal`.
        Some(common_short_val)
    }

    /// Returns the inner u32 value of the `CommonShortVal`.
    pub fn value(&self) -> u32 {
        self.value
    }

    /// Returns the index of the `CommonShortVal`.
    pub fn index(&self) -> u8 {
        self.index
    }
}
