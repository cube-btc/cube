use super::flame_tier::flame_tier::FlameTier;

/// ZKTLC script pubkey.
type ScriptPubKey = Vec<u8>;

/// Account flame set.
pub struct Flame {
    /// Flame tier.
    pub flame_tier: FlameTier,

    /// Flame script pubkey.
    pub script_pubkey: ScriptPubKey,
}

impl Flame {
    /// Constructs a new flame.
    pub fn new(flame_tier: FlameTier, script_pubkey: ScriptPubKey) -> Self {
        Self {
            flame_tier,
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

        // 4 Return the bytes.
        bytes
    }

    /// Deserializes the flame from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Option<Flame> {
        // 1 Decode the flame tier.
        let (flame_tier, remaining_bytes) = {
            // 1.1 Check if there are enough bytes for the flame tier length (1 byte).
            if bytes.len() < 1 {
                return None;
            }

            // 1.2 Read the length of the flame tier bytes.
            let flame_tier_len = bytes[0] as usize;

            // 1.3 Check if there are enough bytes for the flame tier.
            if bytes.len() < 1 + flame_tier_len {
                return None;
            }

            // 1.4 Get the flame tier bytes.
            let flame_tier_bytes = &bytes[1..1 + flame_tier_len];

            // 1.5 Parse the flame tier from bytes.
            let flame_tier = FlameTier::from_bytes(flame_tier_bytes)?;

            // 1.6 Return the flame tier and remaining bytes.
            (flame_tier, &bytes[1 + flame_tier_len..])
        };

        // 2 Decode the script pubkey.
        let script_pubkey = {
            // 2.1 Check if there are enough bytes for the script pubkey length (2 bytes).
            if remaining_bytes.len() < 2 {
                return None;
            }

            // 2.2 Read the length of the script pubkey.
            let mut script_pubkey_len_bytes = [0u8; 2];
            script_pubkey_len_bytes.copy_from_slice(&remaining_bytes[0..2]);
            let script_pubkey_len = u16::from_le_bytes(script_pubkey_len_bytes) as usize;

            // 2.3 Check if there are enough bytes for the script pubkey.
            if remaining_bytes.len() < 2 + script_pubkey_len {
                return None;
            }

            // 2.4 Get the script pubkey.
            remaining_bytes[2..2 + script_pubkey_len].to_vec()
        };

        // 3 Construct the flame.
        let flame = Flame {
            flame_tier,
            script_pubkey,
        };

        // 4 Return the flame.
        Some(flame)
    }
}
