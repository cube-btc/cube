use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Flame version.
type FlameVersion = u8;

/// Flame script pubkey.
type FlameScriptPubKey = Vec<u8>;

/// Tier flame configuration containing version and script pubkey.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TierFlameConfig {
    /// Flame version.
    pub version: FlameVersion,

    /// Script pubkey.
    pub script_pubkey: FlameScriptPubKey,
}

/// Flame config of an account containing various flame-value tiers.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FMAccountFlameConfig {
    // Tier #1 corresponding to 100 satoshis.
    pub tier_1_hundred_satoshis: Option<TierFlameConfig>,

    // Tier #2 corresponding to 1,000 satoshis.
    pub tier_2_thousand_satoshis: Option<TierFlameConfig>,

    // Tier #3 corresponding to 10,000 satoshis.
    pub tier_3_ten_thousand_satoshis: Option<TierFlameConfig>,

    // Tier #4 corresponding to 100,000 satoshis.
    pub tier_4_hundred_thousand_satoshis: Option<TierFlameConfig>,

    // Tier #5 corresponding to 1,000,000 satoshis.
    pub tier_5_one_million_satoshis: Option<TierFlameConfig>,

    // Tier #6 corresponding to 10,000,000 satoshis.
    pub tier_6_ten_million_satoshis: Option<TierFlameConfig>,

    // Tier #7 corresponding to 100,000,000 satoshis.
    pub tier_7_hundred_million_satoshis: Option<TierFlameConfig>,

    // Tier #8 corresponding to any satoshi amount.
    // This tier, if present, becomes the default tier and thus precedes all other 7 tiers.
    // Implemented but intended for potential future use.
    pub tier_any_amount: Option<TierFlameConfig>,
}

impl FMAccountFlameConfig {
    /// Constructs a fresh new flame config.
    pub fn fresh_new() -> Self {
        Self {
            tier_1_hundred_satoshis: None,
            tier_2_thousand_satoshis: None,
            tier_3_ten_thousand_satoshis: None,
            tier_4_hundred_thousand_satoshis: None,
            tier_5_one_million_satoshis: None,
            tier_6_ten_million_satoshis: None,
            tier_7_hundred_million_satoshis: None,
            tier_any_amount: None,
        }
    }

    /// Constructs a new flame config.
    pub fn new(
        // Tier #1
        tier_1: Option<(FlameVersion, FlameScriptPubKey)>,

        // Tier #2
        tier_2: Option<(FlameVersion, FlameScriptPubKey)>,

        // Tier #3
        tier_3: Option<(FlameVersion, FlameScriptPubKey)>,

        // Tier #4
        tier_4: Option<(FlameVersion, FlameScriptPubKey)>,

        // Tier #5
        tier_5: Option<(FlameVersion, FlameScriptPubKey)>,

        // Tier #6
        tier_6: Option<(FlameVersion, FlameScriptPubKey)>,

        // Tier #7
        tier_7: Option<(FlameVersion, FlameScriptPubKey)>,

        // Tier #8
        tier_any_amount: Option<(FlameVersion, FlameScriptPubKey)>,
    ) -> FMAccountFlameConfig {
        // 1 Construct a fresh new flame config.
        let mut flame_config = Self::fresh_new();

        // 2 Match on the tier 1.
        if let Some((version, script_pubkey)) = tier_1 {
            flame_config.tier_1_hundred_satoshis = Some(TierFlameConfig {
                version,
                script_pubkey,
            });
        }

        // 3 Match on the tier 2.
        if let Some((version, script_pubkey)) = tier_2 {
            flame_config.tier_2_thousand_satoshis = Some(TierFlameConfig {
                version,
                script_pubkey,
            });
        }

        // 4 Match on the tier 3.
        if let Some((version, script_pubkey)) = tier_3 {
            flame_config.tier_3_ten_thousand_satoshis = Some(TierFlameConfig {
                version,
                script_pubkey,
            });
        }

        // 5 Match on the tier 4.
        if let Some((version, script_pubkey)) = tier_4 {
            flame_config.tier_4_hundred_thousand_satoshis = Some(TierFlameConfig {
                version,
                script_pubkey,
            });
        }

        // 6 Match on the tier 5.
        if let Some((version, script_pubkey)) = tier_5 {
            flame_config.tier_5_one_million_satoshis = Some(TierFlameConfig {
                version,
                script_pubkey,
            });
        }

        // 7 Match on the tier 6.
        if let Some((version, script_pubkey)) = tier_6 {
            flame_config.tier_6_ten_million_satoshis = Some(TierFlameConfig {
                version,
                script_pubkey,
            });
        }

        // 8 Match on the tier 7.
        if let Some((version, script_pubkey)) = tier_7 {
            flame_config.tier_7_hundred_million_satoshis = Some(TierFlameConfig {
                version,
                script_pubkey,
            });
        }

        // 9 Match on the tier any amount.
        if let Some((version, script_pubkey)) = tier_any_amount {
            flame_config.tier_any_amount = Some(TierFlameConfig {
                version,
                script_pubkey,
            });
        }

        // 10 Return the flame config.
        flame_config
    }

    /// Checks if the flame config is ready to be used.
    pub fn is_ready(&self) -> bool {
        // 1 Check if any of the first 5 tiers are set.
        if self.tier_1_hundred_satoshis.is_some()
            || self.tier_2_thousand_satoshis.is_some()
            || self.tier_3_ten_thousand_satoshis.is_some()
            || self.tier_4_hundred_thousand_satoshis.is_some()
            || self.tier_5_one_million_satoshis.is_some()
        {
            // 1.1 Any of the first 5 tiers are set. Return true.
            return true;
        }

        // 1.2 Otherwise, return false.
        false
    }

    /// Serializes flame config to bytes for the database.
    pub fn to_db_value_bytes(&self) -> Vec<u8> {
        // 1 Construct the bytes.
        let mut bytes = Vec::<u8>::new();

        // 2 Match on the tier 1 hundred satoshis.
        match &self.tier_1_hundred_satoshis {
            // 2.1 If set, push 0x01, serialize the version, and serialize the script pubkey.
            Some(tier_config) => {
                bytes.push(0x01);
                bytes.push(tier_config.version);
                bytes.extend_from_slice(&((tier_config.script_pubkey.len() as u16).to_le_bytes()));
                bytes.extend_from_slice(&tier_config.script_pubkey);
            }
            // 2.2 If not set, push 0x00.
            None => {
                bytes.push(0x00);
            }
        }

        // 3 Match on the tier 2 thousand satoshis.
        match &self.tier_2_thousand_satoshis {
            // 3.1 If set, push 0x01, serialize the version, and serialize the script pubkey.
            Some(tier_config) => {
                bytes.push(0x01);
                bytes.push(tier_config.version);
                bytes.extend_from_slice(&((tier_config.script_pubkey.len() as u16).to_le_bytes()));
                bytes.extend_from_slice(&tier_config.script_pubkey);
            }
            // 3.2 If not set, push 0x00.
            None => {
                bytes.push(0x00);
            }
        }

        // 4 Match on the tier 3 ten thousand satoshis.
        match &self.tier_3_ten_thousand_satoshis {
            // 4.1 If set, push 0x01, serialize the version, and serialize the script pubkey.
            Some(tier_config) => {
                bytes.push(0x01);
                bytes.push(tier_config.version);
                bytes.extend_from_slice(&((tier_config.script_pubkey.len() as u16).to_le_bytes()));
                bytes.extend_from_slice(&tier_config.script_pubkey);
            }
            // 4.2 If not set, push 0x00.
            None => {
                bytes.push(0x00);
            }
        }

        // 5 Match on the tier 4 hundred thousand satoshis.
        match &self.tier_4_hundred_thousand_satoshis {
            // 5.1 If set, push 0x01, serialize the version, and serialize the script pubkey.
            Some(tier_config) => {
                bytes.push(0x01);
                bytes.push(tier_config.version);
                bytes.extend_from_slice(&((tier_config.script_pubkey.len() as u16).to_le_bytes()));
                bytes.extend_from_slice(&tier_config.script_pubkey);
            }
            // 5.2 If not set, push 0x00.
            None => {
                bytes.push(0x00);
            }
        }

        // 6 Match on the tier 5 one million satoshis.
        match &self.tier_5_one_million_satoshis {
            // 6.1 If set, push 0x01, serialize the version, and serialize the script pubkey.
            Some(tier_config) => {
                bytes.push(0x01);
                bytes.push(tier_config.version);
                bytes.extend_from_slice(&((tier_config.script_pubkey.len() as u16).to_le_bytes()));
                bytes.extend_from_slice(&tier_config.script_pubkey);
            }
            // 6.2 If not set, push 0x00.
            None => {
                bytes.push(0x00);
            }
        }

        // 7 Match on the tier 6 ten million satoshis.
        match &self.tier_6_ten_million_satoshis {
            // 7.1 If set, push 0x01, serialize the version, and serialize the script pubkey.
            Some(tier_config) => {
                bytes.push(0x01);
                bytes.push(tier_config.version);
                bytes.push(tier_config.script_pubkey.len() as u8);
                bytes.extend_from_slice(&tier_config.script_pubkey);
            }
            // 7.2 If not set, push 0x00.
            None => {
                bytes.push(0x00);
            }
        }

        // 8 Match on the tier 7 hundred million satoshis.
        match &self.tier_7_hundred_million_satoshis {
            // 8.1 If set, push 0x01, serialize the version, and serialize the script pubkey.
            Some(tier_config) => {
                bytes.push(0x01);
                bytes.push(tier_config.version);
                bytes.extend_from_slice(&((tier_config.script_pubkey.len() as u16).to_le_bytes()));
                bytes.extend_from_slice(&tier_config.script_pubkey);
            }
            // 8.2 If not set, push 0x00.
            None => {
                bytes.push(0x00);
            }
        }

        // 9 Match on the tier any amount.
        match &self.tier_any_amount {
            // 9.1 If set, push 0x01, serialize the version, and serialize the script pubkey.
            Some(tier_config) => {
                bytes.push(0x01);
                bytes.push(tier_config.version);
                bytes.extend_from_slice(&((tier_config.script_pubkey.len() as u16).to_le_bytes()));
                bytes.extend_from_slice(&tier_config.script_pubkey);
            }
            // 9.2 If not set, push 0x00.
            None => {
                bytes.push(0x00);
            }
        }

        // Return the bytes.
        bytes
    }

    /// Constructs flame config from database value bytes.
    pub fn from_db_value_bytes(bytes: &[u8]) -> Option<FMAccountFlameConfig> {
        // 1 Create a cursor to track position in the bytes.
        let mut cursor = 0;

        // 2 Read the tier 1 hundred satoshis.
        let tier_1_hundred_satoshis = match bytes.get(cursor) {
            // 2.1 If set, read the version, length and script pubkey.
            Some(&0x01) => {
                cursor += 1;
                if cursor >= bytes.len() {
                    return None;
                }
                let version = bytes[cursor];
                cursor += 1;
                if cursor + 2 > bytes.len() {
                    return None;
                }
                let length = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
                cursor += 2;
                if cursor + length > bytes.len() {
                    return None;
                }
                let script_pubkey = bytes[cursor..cursor + length].to_vec();
                cursor += length;
                Some(TierFlameConfig {
                    version,
                    script_pubkey,
                })
            }
            // 2.2 If not set, skip.
            Some(&0x00) => {
                cursor += 1;
                None
            }
            _ => return None,
        };

        // 3 Read the tier 2 thousand satoshis.
        let tier_2_thousand_satoshis = match bytes.get(cursor) {
            // 3.1 If set, read the version, length and script pubkey.
            Some(&0x01) => {
                cursor += 1;
                if cursor >= bytes.len() {
                    return None;
                }
                let version = bytes[cursor];
                cursor += 1;
                if cursor + 2 > bytes.len() {
                    return None;
                }
                let length = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
                cursor += 2;
                if cursor + length > bytes.len() {
                    return None;
                }
                let script_pubkey = bytes[cursor..cursor + length].to_vec();
                cursor += length;
                Some(TierFlameConfig {
                    version,
                    script_pubkey,
                })
            }
            // 3.2 If not set, skip.
            Some(&0x00) => {
                cursor += 1;
                None
            }
            _ => return None,
        };

        // 4 Read the tier 3 ten thousand satoshis.
        let tier_3_ten_thousand_satoshis = match bytes.get(cursor) {
            // 4.1 If set, read the version, length and script pubkey.
            Some(&0x01) => {
                cursor += 1;
                if cursor >= bytes.len() {
                    return None;
                }
                let version = bytes[cursor];
                cursor += 1;
                if cursor + 2 > bytes.len() {
                    return None;
                }
                let length = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
                cursor += 2;
                if cursor + length > bytes.len() {
                    return None;
                }
                let script_pubkey = bytes[cursor..cursor + length].to_vec();
                cursor += length;
                Some(TierFlameConfig {
                    version,
                    script_pubkey,
                })
            }
            // 4.2 If not set, skip.
            Some(&0x00) => {
                cursor += 1;
                None
            }
            _ => return None,
        };

        // 5 Read the tier 4 hundred thousand satoshis.
        let tier_4_hundred_thousand_satoshis = match bytes.get(cursor) {
            // 5.1 If set, read the version, length and script pubkey.
            Some(&0x01) => {
                cursor += 1;
                if cursor >= bytes.len() {
                    return None;
                }
                let version = bytes[cursor];
                cursor += 1;
                if cursor + 2 > bytes.len() {
                    return None;
                }
                let length = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
                cursor += 2;
                if cursor + length > bytes.len() {
                    return None;
                }
                let script_pubkey = bytes[cursor..cursor + length].to_vec();
                cursor += length;
                Some(TierFlameConfig {
                    version,
                    script_pubkey,
                })
            }
            // 5.2 If not set, skip.
            Some(&0x00) => {
                cursor += 1;
                None
            }
            _ => return None,
        };

        // 6 Read the tier 5 one million satoshis.
        let tier_5_one_million_satoshis = match bytes.get(cursor) {
            // 6.1 If set, read the version, length and script pubkey.
            Some(&0x01) => {
                cursor += 1;
                if cursor >= bytes.len() {
                    return None;
                }
                let version = bytes[cursor];
                cursor += 1;
                if cursor + 2 > bytes.len() {
                    return None;
                }
                let length = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
                cursor += 2;
                if cursor + length > bytes.len() {
                    return None;
                }
                let script_pubkey = bytes[cursor..cursor + length].to_vec();
                cursor += length;
                Some(TierFlameConfig {
                    version,
                    script_pubkey,
                })
            }
            // 6.2 If not set, skip.
            Some(&0x00) => {
                cursor += 1;
                None
            }
            _ => return None,
        };

        // 7 Read the tier 6 ten million satoshis.
        let tier_6_ten_million_satoshis = match bytes.get(cursor) {
            // 7.1 If set, read the version, length and script pubkey.
            Some(&0x01) => {
                cursor += 1;
                if cursor >= bytes.len() {
                    return None;
                }
                let version = bytes[cursor];
                cursor += 1;
                if cursor >= bytes.len() {
                    return None;
                }
                let length = bytes[cursor] as usize;
                cursor += 1;
                if cursor + length > bytes.len() {
                    return None;
                }
                let script_pubkey = bytes[cursor..cursor + length].to_vec();
                cursor += length;
                Some(TierFlameConfig {
                    version,
                    script_pubkey,
                })
            }
            // 7.2 If not set, skip.
            Some(&0x00) => {
                cursor += 1;
                None
            }
            _ => return None,
        };

        // 8 Read the tier 7 hundred million satoshis.
        let tier_7_hundred_million_satoshis = match bytes.get(cursor) {
            // 8.1 If set, read the version, length and script pubkey.
            Some(&0x01) => {
                cursor += 1;
                if cursor >= bytes.len() {
                    return None;
                }
                let version = bytes[cursor];
                cursor += 1;
                if cursor + 2 > bytes.len() {
                    return None;
                }
                let length = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
                cursor += 2;
                if cursor + length > bytes.len() {
                    return None;
                }
                let script_pubkey = bytes[cursor..cursor + length].to_vec();
                cursor += length;
                Some(TierFlameConfig {
                    version,
                    script_pubkey,
                })
            }
            // 8.2 If not set, skip.
            Some(&0x00) => {
                cursor += 1;
                None
            }
            _ => return None,
        };

        // 9 Read the tier any amount.
        let tier_any_amount = match bytes.get(cursor) {
            // 9.1 If set, read the version, length and script pubkey.
            Some(&0x01) => {
                cursor += 1;
                if cursor >= bytes.len() {
                    return None;
                }
                let version = bytes[cursor];
                cursor += 1;
                if cursor + 2 > bytes.len() {
                    return None;
                }
                let length = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
                cursor += 2;
                if cursor + length > bytes.len() {
                    return None;
                }
                let script_pubkey = bytes[cursor..cursor + length].to_vec();
                cursor += length;
                Some(TierFlameConfig {
                    version,
                    script_pubkey,
                })
            }
            // 9.2 If not set, skip.
            Some(&0x00) => {
                cursor += 1;
                None
            }
            _ => return None,
        };

        // 10 Verify that all bytes were consumed.
        if cursor != bytes.len() {
            return None;
        }

        // 11 Construct the flame config.
        let flame_config = FMAccountFlameConfig {
            tier_1_hundred_satoshis,
            tier_2_thousand_satoshis,
            tier_3_ten_thousand_satoshis,
            tier_4_hundred_thousand_satoshis,
            tier_5_one_million_satoshis,
            tier_6_ten_million_satoshis,
            tier_7_hundred_million_satoshis,
            tier_any_amount,
        };

        // 12 Return the flame config.
        Some(flame_config)
    }

    /// Returns the flame config as a JSON object.
    pub fn json(&self) -> Value {
        // 1 Construct the flame config JSON object.
        let mut obj = Map::new();

        // 2 Insert the tier 1 hundred satoshis.
        obj.insert(
            "tier_1_spk".to_string(),
            match &self.tier_1_hundred_satoshis {
                Some(tier_config) => Value::String(hex::encode(&tier_config.script_pubkey)),
                None => Value::Null,
            },
        );
        obj.insert(
            "tier_1_version".to_string(),
            match &self.tier_1_hundred_satoshis {
                Some(tier_config) => Value::Number(tier_config.version.into()),
                None => Value::Null,
            },
        );

        // 3 Insert the tier 2 thousand satoshis.
        obj.insert(
            "tier_2_spk".to_string(),
            match &self.tier_2_thousand_satoshis {
                Some(tier_config) => Value::String(hex::encode(&tier_config.script_pubkey)),
                None => Value::Null,
            },
        );
        obj.insert(
            "tier_2_version".to_string(),
            match &self.tier_2_thousand_satoshis {
                Some(tier_config) => Value::Number(tier_config.version.into()),
                None => Value::Null,
            },
        );

        // 4 Insert the tier 3 ten thousand satoshis.
        obj.insert(
            "tier_3_spk".to_string(),
            match &self.tier_3_ten_thousand_satoshis {
                Some(tier_config) => Value::String(hex::encode(&tier_config.script_pubkey)),
                None => Value::Null,
            },
        );
        obj.insert(
            "tier_3_version".to_string(),
            match &self.tier_3_ten_thousand_satoshis {
                Some(tier_config) => Value::Number(tier_config.version.into()),
                None => Value::Null,
            },
        );

        // 5 Insert the tier 4 hundred thousand satoshis.
        obj.insert(
            "tier_4_spk".to_string(),
            match &self.tier_4_hundred_thousand_satoshis {
                Some(tier_config) => Value::String(hex::encode(&tier_config.script_pubkey)),
                None => Value::Null,
            },
        );
        obj.insert(
            "tier_4_version".to_string(),
            match &self.tier_4_hundred_thousand_satoshis {
                Some(tier_config) => Value::Number(tier_config.version.into()),
                None => Value::Null,
            },
        );

        // 6 Insert the tier 5 one million satoshis.
        obj.insert(
            "tier_5_spk".to_string(),
            match &self.tier_5_one_million_satoshis {
                Some(tier_config) => Value::String(hex::encode(&tier_config.script_pubkey)),
                None => Value::Null,
            },
        );
        obj.insert(
            "tier_5_version".to_string(),
            match &self.tier_5_one_million_satoshis {
                Some(tier_config) => Value::Number(tier_config.version.into()),
                None => Value::Null,
            },
        );

        // 7 Insert the tier 6 ten million satoshis.
        obj.insert(
            "tier_6_spk".to_string(),
            match &self.tier_6_ten_million_satoshis {
                Some(tier_config) => Value::String(hex::encode(&tier_config.script_pubkey)),
                None => Value::Null,
            },
        );
        obj.insert(
            "tier_6_version".to_string(),
            match &self.tier_6_ten_million_satoshis {
                Some(tier_config) => Value::Number(tier_config.version.into()),
                None => Value::Null,
            },
        );

        // 8 Insert the tier 7 hundred million satoshis.
        obj.insert(
            "tier_7_spk".to_string(),
            match &self.tier_7_hundred_million_satoshis {
                Some(tier_config) => Value::String(hex::encode(&tier_config.script_pubkey)),
                None => Value::Null,
            },
        );
        obj.insert(
            "tier_7_version".to_string(),
            match &self.tier_7_hundred_million_satoshis {
                Some(tier_config) => Value::Number(tier_config.version.into()),
                None => Value::Null,
            },
        );

        // 9 Insert the tier any amount.
        obj.insert(
            "tier_any_amount_spk".to_string(),
            match &self.tier_any_amount {
                Some(tier_config) => Value::String(hex::encode(&tier_config.script_pubkey)),
                None => Value::Null,
            },
        );
        obj.insert(
            "tier_any_amount_version".to_string(),
            match &self.tier_any_amount {
                Some(tier_config) => Value::Number(tier_config.version.into()),
                None => Value::Null,
            },
        );

        // 10 Return the JSON object.
        Value::Object(obj)
    }
}
