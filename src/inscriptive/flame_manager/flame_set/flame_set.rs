use crate::inscriptive::flame_manager::flame::flame::Flame;

/// Rollup height where the flame is projected at.
type AtRollupHeight = u64;

/// Set of flames associated with an account.
pub struct FMAccountFlameSet {
    pub flames: Vec<(Flame, AtRollupHeight)>,
}

impl FMAccountFlameSet {
    /// Constructs a new account flame set.
    pub fn new(flames: Vec<(Flame, AtRollupHeight)>) -> Self {
        Self { flames }
    }

    /// Serializes the account flame set to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        // 1 Initialize the bytes.
        let mut bytes = Vec::<u8>::new();

        // 2 Encode the flames vec length.
        {
            // 2.1 Push the length of the flames vec.
            bytes.extend_from_slice(&((self.flames.len() as u32).to_le_bytes()));
        }

        // 3 Encode each flame and rollup height pair.
        for (flame, rollup_height) in &self.flames {
            // 3.1 Encode the flame.
            {
                // 3.1.1 Get the flame bytes.
                let flame_bytes = flame.to_bytes();

                // 3.1.2 Push the length of the flame bytes.
                bytes.extend_from_slice(&((flame_bytes.len() as u16).to_le_bytes()));

                // 3.1.3 Push the flame bytes.
                bytes.extend_from_slice(&flame_bytes);
            }

            // 3.2 Encode the rollup height.
            {
                // 3.2.1 Push the rollup height.
                bytes.extend_from_slice(&rollup_height.to_le_bytes());
            }
        }

        // 4 Return the bytes.
        bytes
    }

    /// Deserializes the account flame set from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Option<FMAccountFlameSet> {
        // 1 Decode the flames vec length.
        let (flames_len, remaining_bytes) = {
            // 1.1 Check if there are enough bytes for the flames vec length (4 bytes).
            if bytes.len() < 4 {
                return None;
            }

            // 1.2 Read the length of the flames vec.
            let mut flames_len_bytes = [0u8; 4];
            flames_len_bytes.copy_from_slice(&bytes[0..4]);
            let flames_len = u32::from_le_bytes(flames_len_bytes) as usize;

            // 1.3 Return the flames vec length and remaining bytes.
            (flames_len, &bytes[4..])
        };

        // 2 Decode each flame and rollup height pair.
        let mut flames = Vec::<(Flame, AtRollupHeight)>::new();
        let mut current_bytes = remaining_bytes;

        for _ in 0..flames_len {
            // 2.1 Decode the flame.
            let (flame, remaining_after_flame) = {
                // 2.1.1 Check if there are enough bytes for the flame length (2 bytes).
                if current_bytes.len() < 2 {
                    return None;
                }

                // 2.1.2 Read the length of the flame bytes.
                let mut flame_len_bytes = [0u8; 2];
                flame_len_bytes.copy_from_slice(&current_bytes[0..2]);
                let flame_len = u16::from_le_bytes(flame_len_bytes) as usize;

                // 2.1.3 Check if there are enough bytes for the flame.
                if current_bytes.len() < 2 + flame_len {
                    return None;
                }

                // 2.1.4 Get the flame bytes.
                let flame_bytes = &current_bytes[2..2 + flame_len];

                // 2.1.5 Parse the flame from bytes.
                let flame = Flame::from_bytes(flame_bytes)?;

                // 2.1.6 Return the flame and remaining bytes.
                (flame, &current_bytes[2 + flame_len..])
            };

            // 2.2 Decode the rollup height.
            let (rollup_height, remaining_after_rollup_height) = {
                // 2.2.1 Check if there are enough bytes for the rollup height (8 bytes).
                if remaining_after_flame.len() < 8 {
                    return None;
                }

                // 2.2.2 Read the rollup height.
                let mut rollup_height_bytes = [0u8; 8];
                rollup_height_bytes.copy_from_slice(&remaining_after_flame[0..8]);
                let rollup_height = u64::from_le_bytes(rollup_height_bytes);

                // 2.2.3 Return the rollup height and remaining bytes.
                (rollup_height, &remaining_after_flame[8..])
            };

            // 2.3 Push the flame and rollup height pair.
            flames.push((flame, rollup_height));

            // 2.4 Update current bytes for next iteration.
            current_bytes = remaining_after_rollup_height;
        }

        // 3 Construct the account flame set.
        let account_flame_set = FMAccountFlameSet { flames };

        // 4 Return the account flame set.
        Some(account_flame_set)
    }
}
