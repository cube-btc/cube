use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::inscriptive::flame_manager::flame::{flame::Flame, flame_tier::flame_tier::FlameTier};

/// Satoshi amount.
type SatoshiAmount = u64;

/// Taproot witness program of a ZKTLC.
type ZKTLCScriptPubKey = Vec<u8>;

/// Flame config of an account containing various ZKTLC-value tiers.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FMAccountFlameConfig {
    pub zktlc_tier_1_hundred_satoshis: Option<ZKTLCScriptPubKey>,
    pub zktlc_tier_2_thousand_satoshis: Option<ZKTLCScriptPubKey>,
    pub zktlc_tier_3_ten_thousand_satoshis: Option<ZKTLCScriptPubKey>,
    pub zktlc_tier_4_hundred_thousand_satoshis: Option<ZKTLCScriptPubKey>,
    pub zktlc_tier_5_one_million_satoshis: Option<ZKTLCScriptPubKey>,
    pub zktlc_tier_6_ten_million_satoshis: Option<ZKTLCScriptPubKey>,
    pub zktlc_tier_7_hundred_million_satoshis: Option<ZKTLCScriptPubKey>,
    pub zktlc_tier_any_amount: Option<ZKTLCScriptPubKey>,
}

impl FMAccountFlameConfig {
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

    /// Returns the ZKTLCs to fund from a given satoshi amount.
    pub fn retrieve_flames_to_fund(
        &self,
        target_flame_sum_value_in_satoshis: SatoshiAmount,
        current_flame_set_sum_value_in_satoshis: SatoshiAmount,
    ) -> Option<Vec<Flame>> {
        // 1 Check if all tiers are none.
        if self.zktlc_tier_1_hundred_satoshis.is_none()
            && self.zktlc_tier_2_thousand_satoshis.is_none()
            && self.zktlc_tier_3_ten_thousand_satoshis.is_none()
            && self.zktlc_tier_4_hundred_thousand_satoshis.is_none()
            && self.zktlc_tier_5_one_million_satoshis.is_none()
            && self.zktlc_tier_6_ten_million_satoshis.is_none()
            && self.zktlc_tier_7_hundred_million_satoshis.is_none()
            && self.zktlc_tier_any_amount.is_none()
        {
            // 1.1 All tiers are none. Return none.
            return None;
        }

        // 2 Calculate the delta.
        let delta: SatoshiAmount = match target_flame_sum_value_in_satoshis
            .checked_sub(current_flame_set_sum_value_in_satoshis)
        {
            // 2.1 Delta is none. Return none.
            None => return None,

            // 2.2 Delta is some. Return the delta.
            Some(delta) => delta,
        };

        // 3 Initialize the list of flames to return.
        let mut flames = Vec::<Flame>::new();

        // 4 Match on the ZKTLC tier any amount.
        match &self.zktlc_tier_any_amount {
            // 4.a Any amount tier is set.
            Some(script_pubkey) => {
                // 4.a.1 Construct the flame tier.
                let flame_tier = FlameTier::TierAnyAmount(delta);

                // 4.a.2 Construct the flame.
                let flame = Flame::new(flame_tier, script_pubkey.to_owned());

                // 4.a.3 Push the flame to the list of flames to return.
                flames.push(flame);
            }
            // 4.b Any amount tier is not set. We have to push a set of flames.
            None => {
                // 4.b.1 Collect available tiers with their values and script pubkeys.
                // We'll store them as (tier_value, script_pubkey, tier_enum) tuples.
                let mut available_tiers: Vec<(SatoshiAmount, ZKTLCScriptPubKey, FlameTier)> =
                    Vec::new();

                // 4.b.1.1 Check tier 7 (hundred million satoshis = 100000000).
                if let Some(script_pubkey) = &self.zktlc_tier_7_hundred_million_satoshis {
                    available_tiers.push((
                        100000000,
                        script_pubkey.clone(),
                        FlameTier::Tier7HundredMillionSatoshis,
                    ));
                }

                // 4.b.1.2 Check tier 6 (ten million satoshis = 10000000).
                if let Some(script_pubkey) = &self.zktlc_tier_6_ten_million_satoshis {
                    available_tiers.push((
                        10000000,
                        script_pubkey.clone(),
                        FlameTier::Tier6TenMillionSatoshis,
                    ));
                }

                // 4.b.1.3 Check tier 5 (one million satoshis = 1000000).
                if let Some(script_pubkey) = &self.zktlc_tier_5_one_million_satoshis {
                    available_tiers.push((
                        1000000,
                        script_pubkey.clone(),
                        FlameTier::Tier5HundredThousandSatoshis,
                    ));
                }

                // 4.b.1.4 Check tier 4 (hundred thousand satoshis = 100000).
                if let Some(script_pubkey) = &self.zktlc_tier_4_hundred_thousand_satoshis {
                    available_tiers.push((
                        100000,
                        script_pubkey.clone(),
                        FlameTier::Tier4TenThousandSatoshis,
                    ));
                }

                // 4.b.1.5 Check tier 3 (ten thousand satoshis = 10000).
                if let Some(script_pubkey) = &self.zktlc_tier_3_ten_thousand_satoshis {
                    available_tiers.push((
                        10000,
                        script_pubkey.clone(),
                        FlameTier::Tier3ThousandSatoshis,
                    ));
                }

                // 4.b.1.6 Check tier 2 (thousand satoshis = 1000).
                if let Some(script_pubkey) = &self.zktlc_tier_2_thousand_satoshis {
                    available_tiers.push((
                        1000,
                        script_pubkey.clone(),
                        FlameTier::Tier2ThousandSatoshis,
                    ));
                }

                // 4.b.1.7 Check tier 1 (hundred satoshis = 100).
                if let Some(script_pubkey) = &self.zktlc_tier_1_hundred_satoshis {
                    available_tiers.push((
                        100,
                        script_pubkey.clone(),
                        FlameTier::Tier1HundredSatoshis,
                    ));
                }

                // 4.b.1.8 If no tiers are available, return None.
                if available_tiers.is_empty() {
                    return None;
                }

                // 4.b.2 Sort tiers in descending order by value (largest first).
                available_tiers.sort_by(|a, b| b.0.cmp(&a.0));

                // 4.b.2.1 Store the smallest tier for rounding up if needed.
                let smallest_tier = available_tiers.last().cloned();

                // 4.b.3 Greedily select tiers to round up the delta.
                let mut remaining_delta = delta;
                for (tier_value, script_pubkey, flame_tier) in &available_tiers {
                    // 4.b.3.1 Calculate how many of this tier we need.
                    let count = remaining_delta / tier_value;

                    // 4.b.3.2 If we need at least one of this tier, add them.
                    if count > 0 {
                        for _ in 0..count {
                            let flame = Flame::new(*flame_tier, script_pubkey.clone());
                            flames.push(flame);
                        }
                        remaining_delta -= count * tier_value;
                    }

                    // 4.b.3.3 If we've covered the delta, we're done.
                    if remaining_delta == 0 {
                        break;
                    }
                }

                // 4.b.4 If there's still remaining delta, we need to round up by adding one more of the smallest available tier.
                if remaining_delta > 0 {
                    // 4.b.4.1 Use the smallest available tier we stored earlier.
                    if let Some((_, script_pubkey, flame_tier)) = smallest_tier {
                        let flame = Flame::new(flame_tier, script_pubkey);
                        flames.push(flame);
                    }
                }
            }
        }

        // 5 Return the list of flames to return.
        Some(flames)
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
    pub fn from_db_value_bytes(bytes: &[u8]) -> Option<FMAccountFlameConfig> {
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
        let flame_config = FMAccountFlameConfig {
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
