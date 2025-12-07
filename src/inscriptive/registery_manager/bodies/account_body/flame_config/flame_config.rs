use serde_json::{Map, Value};

/// Satoshi amount.
type SatoshiAmount = u64;

/// Taproot witness program of a ZKTLC.
type ZKTLCScriptPubKey = Vec<u8>;

/// Flame config of an account containing various ZKTLC-value tiers.
#[derive(Clone)]
pub struct FlameConfig {
    pub zktlc_tier_1_hundred_satoshis: Option<ZKTLCScriptPubKey>,
    pub zktlc_tier_2_thousand_satoshis: Option<ZKTLCScriptPubKey>,
    pub zktlc_tier_3_ten_thousand_satoshis: Option<ZKTLCScriptPubKey>,
    pub zktlc_tier_4_hundred_thousand_satoshis: Option<ZKTLCScriptPubKey>,
    pub zktlc_tier_5_one_million_satoshis: Option<ZKTLCScriptPubKey>,
    pub zktlc_tier_6_ten_million_satoshis: Option<ZKTLCScriptPubKey>,
    pub zktlc_tier_7_hundred_million_satoshis: Option<ZKTLCScriptPubKey>,
    pub zktlc_tier_any_amount: Option<ZKTLCScriptPubKey>,
}

impl FlameConfig {
    /// Constructs a fresh new flame config.
    pub fn fresh_new() -> Self {
        Self {
            zktlc_tier_1_hundred_satoshis: None,
            zktlc_tier_2_thousand_satoshis: None,
            zktlc_tier_3_ten_thousand_satoshis: None,
            zktlc_tier_4_hundred_thousand_satoshis: None,
            zktlc_tier_5_one_million_satoshis: None,
            zktlc_tier_6_ten_million_satoshis: None,
            zktlc_tier_7_hundred_million_satoshis: None,
            zktlc_tier_any_amount: None,
        }
    }

    /// Serializes flame config to bytes for the database.
    pub fn to_db_value_bytes(&self) -> Vec<u8> {
        // 1 Construct the bytes.
        let mut bytes = Vec::<u8>::new();

        // 2 Match on the ZKTLC tier 1 hundred satoshis.
        match &self.zktlc_tier_1_hundred_satoshis {
            // 2.1 If set, push 0x01 and serialize the script pubkey.
            Some(script_pubkey) => {
                bytes.push(0x01);
                bytes.extend_from_slice(&((script_pubkey.len() as u16).to_le_bytes()));
                bytes.extend_from_slice(&script_pubkey);
            }
            // 2.2 If not set, push 0x00.
            None => {
                bytes.push(0x00);
            }
        }

        // 3 Match on the ZKTLC tier 2 thousand satoshis.
        match &self.zktlc_tier_2_thousand_satoshis {
            // 3.1 If set, push 0x01 and serialize the script pubkey.
            Some(script_pubkey) => {
                bytes.push(0x01);
                bytes.extend_from_slice(&((script_pubkey.len() as u16).to_le_bytes()));
                bytes.extend_from_slice(&script_pubkey);
            }
            // 3.2 If not set, push 0x00.
            None => {
                bytes.push(0x00);
            }
        }

        // 4 Match on the ZKTLC tier 3 ten thousand satoshis.
        match &self.zktlc_tier_3_ten_thousand_satoshis {
            // 4.1 If set, push 0x01 and serialize the script pubkey.
            Some(script_pubkey) => {
                bytes.push(0x01);
                bytes.extend_from_slice(&((script_pubkey.len() as u16).to_le_bytes()));
                bytes.extend_from_slice(&script_pubkey);
            }
            // 4.2 If not set, push 0x00.
            None => {
                bytes.push(0x00);
            }
        }

        // 5 Match on the ZKTLC tier 4 hundred thousand satoshis.
        match &self.zktlc_tier_4_hundred_thousand_satoshis {
            // 5.1 If set, push 0x01 and serialize the script pubkey.
            Some(script_pubkey) => {
                bytes.push(0x01);
                bytes.extend_from_slice(&((script_pubkey.len() as u16).to_le_bytes()));
                bytes.extend_from_slice(&script_pubkey);
            }
            // 5.2 If not set, push 0x00.
            None => {
                bytes.push(0x00);
            }
        }

        // 6 Match on the ZKTLC tier 5 one million satoshis.
        match &self.zktlc_tier_5_one_million_satoshis {
            // 6.1 If set, push 0x01 and serialize the script pubkey.
            Some(script_pubkey) => {
                bytes.push(0x01);
                bytes.extend_from_slice(&((script_pubkey.len() as u16).to_le_bytes()));
                bytes.extend_from_slice(&script_pubkey);
            }
            // 6.2 If not set, push 0x00.
            None => {
                bytes.push(0x00);
            }
        }

        // 7 Match on the ZKTLC tier 6 ten million satoshis.
        match &self.zktlc_tier_6_ten_million_satoshis {
            // 7.1 If set, push 0x01 and serialize the script pubkey.
            Some(script_pubkey) => {
                bytes.push(0x01);
                bytes.push(script_pubkey.len() as u8);
                bytes.extend_from_slice(&script_pubkey);
            }
            // 7.2 If not set, push 0x00.
            None => {
                bytes.push(0x00);
            }
        }

        // 8 Match on the ZKTLC tier 7 hundred million satoshis.
        match &self.zktlc_tier_7_hundred_million_satoshis {
            // 8.1 If set, push 0x01 and serialize the script pubkey.
            Some(script_pubkey) => {
                bytes.push(0x01);
                bytes.extend_from_slice(&((script_pubkey.len() as u16).to_le_bytes()));
                bytes.extend_from_slice(&script_pubkey);
            }
            // 8.2 If not set, push 0x00.
            None => {
                bytes.push(0x00);
            }
        }

        // 9 Match on the ZKTLC tier any amount.
        match &self.zktlc_tier_any_amount {
            // 9.1 If set, push 0x01 and serialize the script pubkey.
            Some(script_pubkey) => {
                bytes.push(0x01);
                bytes.extend_from_slice(&((script_pubkey.len() as u16).to_le_bytes()));
                bytes.extend_from_slice(&script_pubkey);
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
    pub fn from_db_value_bytes(bytes: &[u8]) -> Option<FlameConfig> {
        // 1 Create a cursor to track position in the bytes.
        let mut cursor = 0;

        // 2 Read the ZKTLC tier 1 hundred satoshis.
        let zktlc_tier_1_hundred_satoshis = match bytes.get(cursor) {
            // 2.1 If set, read the length and script pubkey.
            Some(&0x01) => {
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
                Some(script_pubkey)
            }
            // 2.2 If not set, skip.
            Some(&0x00) => {
                cursor += 1;
                None
            }
            _ => return None,
        };

        // 3 Read the ZKTLC tier 2 thousand satoshis.
        let zktlc_tier_2_thousand_satoshis = match bytes.get(cursor) {
            // 3.1 If set, read the length and script pubkey.
            Some(&0x01) => {
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
                Some(script_pubkey)
            }
            // 3.2 If not set, skip.
            Some(&0x00) => {
                cursor += 1;
                None
            }
            _ => return None,
        };

        // 4 Read the ZKTLC tier 3 ten thousand satoshis.
        let zktlc_tier_3_ten_thousand_satoshis = match bytes.get(cursor) {
            // 4.1 If set, read the length and script pubkey.
            Some(&0x01) => {
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
                Some(script_pubkey)
            }
            // 4.2 If not set, skip.
            Some(&0x00) => {
                cursor += 1;
                None
            }
            _ => return None,
        };

        // 5 Read the ZKTLC tier 4 hundred thousand satoshis.
        let zktlc_tier_4_hundred_thousand_satoshis = match bytes.get(cursor) {
            // 5.1 If set, read the length and script pubkey.
            Some(&0x01) => {
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
                Some(script_pubkey)
            }
            // 5.2 If not set, skip.
            Some(&0x00) => {
                cursor += 1;
                None
            }
            _ => return None,
        };

        // 6 Read the ZKTLC tier 5 one million satoshis.
        let zktlc_tier_5_one_million_satoshis = match bytes.get(cursor) {
            // 6.1 If set, read the length and script pubkey.
            Some(&0x01) => {
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
                Some(script_pubkey)
            }
            // 6.2 If not set, skip.
            Some(&0x00) => {
                cursor += 1;
                None
            }
            _ => return None,
        };

        // 7 Read the ZKTLC tier 6 ten million satoshis.
        let zktlc_tier_6_ten_million_satoshis = match bytes.get(cursor) {
            // 7.1 If set, read the length and script pubkey.
            Some(&0x01) => {
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
                Some(script_pubkey)
            }
            // 7.2 If not set, skip.
            Some(&0x00) => {
                cursor += 1;
                None
            }
            _ => return None,
        };

        // 8 Read the ZKTLC tier 7 hundred million satoshis.
        let zktlc_tier_7_hundred_million_satoshis = match bytes.get(cursor) {
            // 8.1 If set, read the length and script pubkey.
            Some(&0x01) => {
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
                Some(script_pubkey)
            }
            // 8.2 If not set, skip.
            Some(&0x00) => {
                cursor += 1;
                None
            }
            _ => return None,
        };

        // 9 Read the ZKTLC tier any amount.
        let zktlc_tier_any_amount = match bytes.get(cursor) {
            // 9.1 If set, read the length and script pubkey.
            Some(&0x01) => {
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
                Some(script_pubkey)
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
        let flame_config = FlameConfig {
            zktlc_tier_1_hundred_satoshis,
            zktlc_tier_2_thousand_satoshis,
            zktlc_tier_3_ten_thousand_satoshis: zktlc_tier_3_ten_thousand_satoshis,
            zktlc_tier_4_hundred_thousand_satoshis,
            zktlc_tier_5_one_million_satoshis,
            zktlc_tier_6_ten_million_satoshis,
            zktlc_tier_7_hundred_million_satoshis: zktlc_tier_7_hundred_million_satoshis,
            zktlc_tier_any_amount: zktlc_tier_any_amount,
        };

        // 12 Return the flame config.
        Some(flame_config)
    }

    /// Returns the ZKTLCs to fund from a given satoshi amount.
    pub fn zktlcs_to_fund_from_amount(
        &self,
        amount: SatoshiAmount,
    ) -> Vec<(SatoshiAmount, ZKTLCScriptPubKey)> {
        // If any amount exists, directly return it.
        match &self.zktlc_tier_any_amount {
            // If any amount exists, directly return it.
            Some(script_pubkey) => vec![(amount, script_pubkey.to_owned())],
            // Otherwise, run an algorithm to find the most efficient ZKTLCs with rounding up.
            None => {
                // TODO!!!
                // For now, return an empty vector.
                Vec::<(SatoshiAmount, ZKTLCScriptPubKey)>::new()
            }
        }
    }

    /// Returns the flame config as a JSON object.
    pub fn json(&self) -> Value {
        // 1 Construct the flame config JSON object.
        let mut obj = Map::new();

        // 2 Insert the ZKTLC tier 1 hundred satoshis.
        obj.insert(
            "zktlc_tier_1_spk".to_string(),
            match &self.zktlc_tier_1_hundred_satoshis {
                Some(script_pubkey) => Value::String(hex::encode(script_pubkey)),
                None => Value::Null,
            },
        );

        // 3 Insert the ZKTLC tier 2 thousand satoshis.
        obj.insert(
            "zktlc_tier_2_spk".to_string(),
            match &self.zktlc_tier_2_thousand_satoshis {
                Some(script_pubkey) => Value::String(hex::encode(script_pubkey)),
                None => Value::Null,
            },
        );

        // 4 Insert the ZKTLC tier 3 ten thousand satoshis.
        obj.insert(
            "zktlc_tier_3_spk".to_string(),
            match &self.zktlc_tier_3_ten_thousand_satoshis {
                Some(script_pubkey) => Value::String(hex::encode(script_pubkey)),
                None => Value::Null,
            },
        );

        // 5 Insert the ZKTLC tier 4 hundred thousand satoshis.
        obj.insert(
            "zktlc_tier_4_spk".to_string(),
            match &self.zktlc_tier_4_hundred_thousand_satoshis {
                Some(script_pubkey) => Value::String(hex::encode(script_pubkey)),
                None => Value::Null,
            },
        );

        // 6 Insert the ZKTLC tier 5 one million satoshis.
        obj.insert(
            "zktlc_tier_5_spk".to_string(),
            match &self.zktlc_tier_5_one_million_satoshis {
                Some(script_pubkey) => Value::String(hex::encode(script_pubkey)),
                None => Value::Null,
            },
        );

        // 7 Insert the ZKTLC tier 6 ten million satoshis.
        obj.insert(
            "zktlc_tier_6_spk".to_string(),
            match &self.zktlc_tier_6_ten_million_satoshis {
                Some(script_pubkey) => Value::String(hex::encode(script_pubkey)),
                None => Value::Null,
            },
        );

        // 8 Insert the ZKTLC tier 7 hundred million satoshis.
        obj.insert(
            "zktlc_tier_7_spk".to_string(),
            match &self.zktlc_tier_7_hundred_million_satoshis {
                Some(script_pubkey) => Value::String(hex::encode(script_pubkey)),
                None => Value::Null,
            },
        );

        // 9 Insert the ZKTLC tier any amount.
        obj.insert(
            "zktlc_tier_any_amount_spk".to_string(),
            match &self.zktlc_tier_any_amount {
                Some(script_pubkey) => Value::String(hex::encode(script_pubkey)),
                None => Value::Null,
            },
        );

        // 10 Return the JSON object.
        Value::Object(obj)
    }
}
