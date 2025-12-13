use super::flame_tier::flame_tier::FlameTier;

/// Rollup height.
type RollupHeight = u64;

/// ZKTLC script pubkey.
type ScriptPubKey = Vec<u8>;

/// Account flame set.
pub struct Flame {
    /// Flame tier.
    pub flame_tier: FlameTier,

    /// ZKTLC script pubkey.
    pub script_pubkey: ScriptPubKey,

    /// Nesting location of the flame.
    /// Where does this flame nest/live ?
    pub nesting_location: Option<RollupHeight>,
}

impl Flame {
    /// Constructs a new flame.
    pub fn new(
        flame_tier: FlameTier,
        script_pubkey: ScriptPubKey,
        nesting_location: Option<RollupHeight>,
    ) -> Self {
        Self {
            flame_tier,
            nesting_location,
            script_pubkey,
        }
    }

    /// Serializes the flame to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        // 1 Initialize the bytes.
        let mut bytes = Vec::<u8>::new();

        // 2 Encode the flame tier.
        {
            // 2.1 Get the flame tier bytes.
            let flame_tier_bytes = self.flame_tier.to_bytes();

            // 2.2 Push the length of the flame tier bytes.
            bytes.extend_from_slice(&(flame_tier_bytes.len() as u8).to_le_bytes());

            // 2.3 Push the flame tier bytes.
            bytes.extend_from_slice(&flame_tier_bytes);
        }

        // 3 Encode the script pubkey.
        {
            // 3.1 Push the length of the script pubkey.
            bytes.extend_from_slice(&((self.script_pubkey.len() as u16).to_le_bytes()));

            // 3.2 Push the script pubkey.
            bytes.extend_from_slice(&self.script_pubkey);
        }

        // 4 Encode the rollup height.
        {
            // 4.1 Match on the nesting location.
            match self.nesting_location {
                // 4.1.a The nesting location is set.
                Some(nesting_location) => {
                    // 4.1.a.1 Push the one byte to tell if the nesting location is set.
                    bytes.push(1);

                    // 4.1.a.2 Push the nesting location as a little-endian u64.
                    bytes.extend_from_slice(&nesting_location.to_le_bytes());
                }

                // 4.1.b The nesting location is not set.
                None => {
                    // 4.1.b.1 Push the one byte to tell if the nesting location is not set.
                    bytes.push(0);
                }
            }
        }

        // 5 Return the bytes.
        bytes
    }

    /// Deserializes the flame from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Option<Flame> {
        // 1 Initialize the cursor.
        let mut cursor = 0;

        // 2 Decode the flame tier.
        let flame_tier = {
            // 2.1 Check if there are enough bytes for the flame tier length.
            if cursor + 1 > bytes.len() {
                return None;
            }

            // 2.2 Read the length of the flame tier bytes.
            let flame_tier_length = bytes[cursor] as usize;
            cursor += 1;

            // 2.3 Check if there are enough bytes for the flame tier.
            if cursor + flame_tier_length > bytes.len() {
                return None;
            }

            // 2.4 Read the flame tier bytes and deserialize.
            let flame_tier_bytes = &bytes[cursor..cursor + flame_tier_length];
            let flame_tier = FlameTier::from_bytes(flame_tier_bytes)?;
            cursor += flame_tier_length;

            flame_tier
        };

        // 3 Decode the script pubkey.
        let script_pubkey = {
            // 3.1 Check if there are enough bytes for the script pubkey length.
            if cursor + 2 > bytes.len() {
                return None;
            }

            // 3.2 Read the length of the script pubkey.
            let script_pubkey_length =
                u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
            cursor += 2;

            // 3.3 Check if there are enough bytes for the script pubkey.
            if cursor + script_pubkey_length > bytes.len() {
                return None;
            }

            // 3.4 Read the script pubkey.
            let script_pubkey = bytes[cursor..cursor + script_pubkey_length].to_vec();
            cursor += script_pubkey_length;

            script_pubkey
        };

        // 4 Decode the nesting location.
        let nesting_location = {
            // 4.1 Check if there are enough bytes for the one byte to tell if the nesting location is set.
            if cursor + 1 > bytes.len() {
                return None;
            }

            // 4.2 Read the one byte to tell if the nesting location is set or not.
            let nesting_location_flag = bytes[cursor];
            cursor += 1;

            // 4.3 Match on the one byte to tell if the nesting location is set or not.
            match nesting_location_flag {
                // 4.3.a The nesting location is set.
                1 => {
                    // 4.3.a.1 Check if there are enough bytes for the nesting location as a little-endian u64 (8 bytes).
                    if cursor + 8 > bytes.len() {
                        return None;
                    }

                    // 4.3.a.2 Read the nesting location as a little-endian u64 (8 bytes).
                    let mut nesting_location_bytes = [0u8; 8];
                    nesting_location_bytes.copy_from_slice(&bytes[cursor..cursor + 8]);

                    // 4.3.a.3 Return the nesting location as a little-endian u64 (8 bytes).
                    Some(u64::from_le_bytes(nesting_location_bytes))
                }

                // 4.3.b The nesting location is not set.
                0 => None,

                // 4.3.c Invalid flag value.
                _ => return None,
            }
        };

        // 5 Construct the flame.
        let flame = Flame {
            flame_tier,
            script_pubkey,
            nesting_location,
        };

        // 6 Return the flame.
        Some(flame)
    }
}
