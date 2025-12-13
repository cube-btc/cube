use serde::{Deserialize, Serialize};

/// Satoshi amount.
type SatoshiAmount = u64;

/// Flame tier.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlameTier {
    Tier1HundredSatoshis,
    Tier2ThousandSatoshis,
    Tier3ThousandSatoshis,
    Tier4TenThousandSatoshis,
    Tier5HundredThousandSatoshis,
    Tier6TenMillionSatoshis,
    Tier7HundredMillionSatoshis,
    TierAnyAmount(SatoshiAmount),
}

impl FlameTier {
    /// Constructs a new flame tier.
    pub fn new(satoshi_amount: u64) -> Self {
        match satoshi_amount {
            100 => FlameTier::Tier1HundredSatoshis,
            1000 => FlameTier::Tier2ThousandSatoshis,
            10000 => FlameTier::Tier3ThousandSatoshis,
            100000 => FlameTier::Tier4TenThousandSatoshis,
            1000000 => FlameTier::Tier5HundredThousandSatoshis,
            10000000 => FlameTier::Tier6TenMillionSatoshis,
            100000000 => FlameTier::Tier7HundredMillionSatoshis,
            _ => FlameTier::TierAnyAmount(satoshi_amount),
        }
    }
    /// Returns the satoshi amount.
    pub fn satoshi_amount(&self) -> u64 {
        match self {
            FlameTier::Tier1HundredSatoshis => 100,
            FlameTier::Tier2ThousandSatoshis => 1000,
            FlameTier::Tier3ThousandSatoshis => 10000,
            FlameTier::Tier4TenThousandSatoshis => 100000,
            FlameTier::Tier5HundredThousandSatoshis => 1000000,
            FlameTier::Tier6TenMillionSatoshis => 10000000,
            FlameTier::Tier7HundredMillionSatoshis => 100000000,
            FlameTier::TierAnyAmount(amount) => *amount,
        }
    }

    /// Serializes the flame tier to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        // 1 Match on the flame tier.
        match self {
            // 1.a If the flame tier is tier 1 hundred satoshis.
            FlameTier::Tier1HundredSatoshis => vec![0x01],

            // 1.b If the flame tier is tier 2 thousand satoshis.
            FlameTier::Tier2ThousandSatoshis => vec![0x02],

            // 1.c If the flame tier is tier 3 ten thousand satoshis.
            FlameTier::Tier3ThousandSatoshis => vec![0x03],

            // 1.d If the flame tier is tier 4 hundred thousand satoshis.
            FlameTier::Tier4TenThousandSatoshis => vec![0x04],

            // 1.e If the flame tier is tier 5 one million satoshis.
            FlameTier::Tier5HundredThousandSatoshis => vec![0x05],

            // 1.f If the flame tier is tier 6 ten million satoshis.
            FlameTier::Tier6TenMillionSatoshis => vec![0x06],

            // 1.g If the flame tier is tier 7 hundred million satoshis.
            FlameTier::Tier7HundredMillionSatoshis => vec![0x07],

            // 1.h If the flame tier is tier any amount.
            FlameTier::TierAnyAmount(amount) => {
                // 1.h.a Initialize the bytes.
                let mut bytes = Vec::<u8>::new();

                // 1.h.b Push the 0x08.
                bytes.push(0x08);

                // 1.h.c Push the amount.
                bytes.extend_from_slice(&(*amount).to_le_bytes());

                // 1.h.d Return the bytes.
                bytes
            }
        }
    }

    /// Deserializes the flame tier from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Option<FlameTier> {
        // 1 Check if the bytes are empty.
        if bytes.is_empty() {
            return None;
        }

        // 2 Match on the first byte.
        match bytes[0] {
            // 2.a If the first byte is 0x01, return tier 1 hundred satoshis.
            0x01 => Some(FlameTier::Tier1HundredSatoshis),

            // 2.b If the first byte is 0x02, return tier 2 thousand satoshis.
            0x02 => Some(FlameTier::Tier2ThousandSatoshis),

            // 2.c If the first byte is 0x03, return tier 3 ten thousand satoshis.
            0x03 => Some(FlameTier::Tier3ThousandSatoshis),

            // 2.d If the first byte is 0x04, return tier 4 hundred thousand satoshis.
            0x04 => Some(FlameTier::Tier4TenThousandSatoshis),

            // 2.e If the first byte is 0x05, return tier 5 one million satoshis.
            0x05 => Some(FlameTier::Tier5HundredThousandSatoshis),

            // 2.f If the first byte is 0x06, return tier 6 ten million satoshis.
            0x06 => Some(FlameTier::Tier6TenMillionSatoshis),

            // 2.g If the first byte is 0x07, return tier 7 hundred million satoshis.
            0x07 => Some(FlameTier::Tier7HundredMillionSatoshis),

            // 2.h If the first byte is 0x08, parse tier any amount.
            0x08 => {
                // 2.h.a Check if there are enough bytes for a u64 (8 bytes).
                if bytes.len() < 9 {
                    return None;
                }

                // 2.h.b Read the amount as a little-endian u64.
                let mut amount_bytes = [0u8; 8];
                amount_bytes.copy_from_slice(&bytes[1..9]);
                let amount = u64::from_le_bytes(amount_bytes);

                // 2.h.c Return tier any amount with the parsed amount.
                Some(FlameTier::TierAnyAmount(amount))
            }
            // 2.i Otherwise, return None (invalid byte).
            _ => None,
        }
    }
}
