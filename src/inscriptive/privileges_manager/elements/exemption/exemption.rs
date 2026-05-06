use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use crate::inscriptive::privileges_manager::elements::exemption::periodic_resource::periodic_resource::PeriodicResource;

/// Remaining fee (satoshis) after each exemption stage, for analysis and batch records.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct ExemptionSubsidyBreakdown {
    /// Amount still owed after periodic credit is applied.
    pub post_periodic_credit_leftover: u64,
    /// Amount still owed after direct credit is applied.
    pub post_direct_credit_leftover: u64,
    /// Final amount still owed after the discount fraction is applied.
    pub post_discount_leftover: u64,
}

impl ExemptionSubsidyBreakdown {
    pub fn json(&self) -> Value {
        let mut obj = Map::new();
        obj.insert(
            "post_periodic_credit".to_string(),
            Value::Number(self.post_periodic_credit_leftover.into()),
        );
        obj.insert(
            "post_direct_credit".to_string(),
            Value::Number(self.post_direct_credit_leftover.into()),
        );
        obj.insert(
            "post_discount_final".to_string(),
            Value::Number(self.post_discount_leftover.into()),
        );
        Value::Object(obj)
    }
}

/// A struct for repreferring to the exemption of a resource, transaction fees for the accounts, and occupancy tax for the contracts.
#[derive(Clone, Serialize, Deserialize)]
pub struct Exemption {
    // Primary, periodic, refilling credit.
    pub periodic_credit: Option<(PeriodicResource, u64)>,

    // Secondary, direct, non-refilling credit, applied after the periodic credit, on top of the remaining amount.
    pub direct_credit: Option<(u64, u64)>,

    // Third place, discount: fraction `discount / 200` of the post-direct leftover is waived (not depleted). Values above 200 are treated as 200 when applying.
    pub discount: Option<(u8, u64)>,
}

impl Exemption {
    // Constructs a fresh new exemption instance.
    pub fn new(
        periodic_credit: Option<(PeriodicResource, u64)>,
        direct_credit: Option<(u64, u64)>,
        discount: Option<(u8, u64)>,
    ) -> Self {
        Self {
            periodic_credit,
            direct_credit,
            discount,
        }
    }

    /// Serializes exemption bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        // 1 Create an empty byte vector.
        let mut bytes = Vec::<u8>::new();

        // 2 Serialize presence flags.
        bytes.push(u8::from(self.periodic_credit.is_some()));
        bytes.push(u8::from(self.direct_credit.is_some()));
        bytes.push(u8::from(self.discount.is_some()));

        // 3 Serialize periodic exemption if present.
        if let Some((periodic_credit, periodic_credit_expiry_timestamp)) = &self.periodic_credit {
            bytes.extend(periodic_credit.to_bytes());
            bytes.extend(periodic_credit_expiry_timestamp.to_le_bytes());
        }

        // 4 Serialize direct exemption if present.
        if let Some((direct_credit, direct_credit_expiry_timestamp)) = &self.direct_credit {
            bytes.extend(direct_credit.to_le_bytes());
            bytes.extend(direct_credit_expiry_timestamp.to_le_bytes());
        }

        // 5 Serialize discount and expiry if present.
        if let Some((discount, discount_expiry_timestamp)) = &self.discount {
            bytes.push(*discount);
            bytes.extend(discount_expiry_timestamp.to_le_bytes());
        }

        // 6 Return the bytes.
        bytes
    }

    /// Deserializes exemption bytes from:
    pub fn from_bytes(bytes: &[u8]) -> Option<Exemption> {
        // 1 We need at least 3 bytes for presence flags.
        if bytes.len() < 3 {
            return None;
        }

        // 2 Read presence flags in the same order as serialization.
        let periodic_credit_present = bytes[0] != 0;
        let direct_credit_present = bytes[1] != 0;
        let discount_present = bytes[2] != 0;

        // 3 Start reading payload right after flags.
        let mut cursor = 3usize;

        // 4 Deserialize periodic credit and expiry if present.
        let periodic_credit = if periodic_credit_present {
            if bytes.len() < cursor + 24 + 8 {
                return None;
            }

            let periodic_credit = PeriodicResource::from_bytes(&bytes[cursor..cursor + 24])?;
            cursor += 24;

            let periodic_credit_expiry_timestamp =
                u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().ok()?);
            cursor += 8;

            Some((periodic_credit, periodic_credit_expiry_timestamp))
        } else {
            None
        };

        // 5 Deserialize direct credit and expiry if present.
        let direct_credit = if direct_credit_present {
            if bytes.len() < cursor + 8 + 8 {
                return None;
            }

            let direct_credit = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().ok()?);
            cursor += 8;

            let direct_credit_expiry_timestamp =
                u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().ok()?);
            cursor += 8;

            Some((direct_credit, direct_credit_expiry_timestamp))
        } else {
            None
        };

        // 6 Deserialize discount and expiry if present.
        let discount = if discount_present {
            if bytes.len() < cursor + 1 + 8 {
                return None;
            }

            let discount = bytes[cursor];
            cursor += 1;

            let discount_expiry_timestamp =
                u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().ok()?);
            cursor += 8;

            Some((discount, discount_expiry_timestamp))
        } else {
            None
        };

        // 7 Reject extra trailing bytes.
        if cursor != bytes.len() {
            return None;
        }

        // 8 Return decoded exemption.
        Some(Exemption {
            periodic_credit,
            direct_credit,
            discount,
        })
    }

    /// Applies the subsidy to the exemption, then returns per-stage leftovers (final = `post_discount_leftover`).
    pub fn apply_subsidy(
        &mut self,
        current_timestamp: u64,
        latest_activity_timestamp: u64,
        consume_amount: u64,
    ) -> Option<ExemptionSubsidyBreakdown> {
        // 1 Refill and consume periodic credit only if active.
        let post_periodic_credit_leftover = match &mut self.periodic_credit {
            Some((periodic_credit, periodic_expiry)) if current_timestamp <= *periodic_expiry => {
                periodic_credit.refill_and_consume(
                    current_timestamp,
                    latest_activity_timestamp,
                    consume_amount,
                )?
            }
            _ => consume_amount,
        };

        // 2 Subsidize the direct credit only if active.
        let direct_credit = match self.direct_credit {
            Some((direct_credit, direct_expiry)) if current_timestamp <= direct_expiry => direct_credit,
            _ => 0,
        };

        let post_direct_credit_leftover = match direct_credit >= post_periodic_credit_leftover {
            // We can fully subsidize the rest of the direct credit.
            true => {
                let direct_credit_left = direct_credit - post_periodic_credit_leftover;
                if let Some((direct_credit_mut, direct_expiry)) = &mut self.direct_credit {
                    if current_timestamp <= *direct_expiry {
                        *direct_credit_mut = direct_credit_left;
                    }
                }
                0u64
            }
            false => {
                let leftover = post_periodic_credit_leftover - direct_credit;
                if let Some((direct_credit_mut, direct_expiry)) = &mut self.direct_credit {
                    if current_timestamp <= *direct_expiry {
                        *direct_credit_mut = 0;
                    }
                }
                leftover
            }
        };

        // 3 Subsidize discount only if active: waive `floor(leftover * min(discount, 200) / 200)` (discount is not depleted).
        let leftover_u = post_direct_credit_leftover as u128;
        let discount_factor = match self.discount {
            Some((discount, discount_expiry)) if current_timestamp <= discount_expiry => {
                (discount as u128).min(200)
            }
            _ => 0,
        };
        let discount_amount = ((leftover_u * discount_factor) / 200) as u64;
        let post_discount_leftover = post_direct_credit_leftover.saturating_sub(discount_amount);

        Some(ExemptionSubsidyBreakdown {
            post_periodic_credit_leftover,
            post_direct_credit_leftover,
            post_discount_leftover,
        })
    }
}
