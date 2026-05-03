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
    pub periodic_credit: PeriodicResource,

    // Secondary, direct, non-refilling credit, applied after the periodic credit, on top of the remaining amount.
    pub direct_credit: u64,

    // Third place, discount: fraction `discount / 200` of the post-direct leftover is waived (not depleted). Values above 200 are treated as 200 when applying.
    pub discount: u8,
}

impl Exemption {
    // Constructs a fresh new exemption instance.
    pub fn new(periodic_credit: PeriodicResource, direct_credit: u64, discount: u8) -> Self {
        Self {
            periodic_credit,
            direct_credit,
            discount,
        }
    }

    /// Serializes the exemption to bytes (`24` periodic + `8` direct + `1` discount = `33` bytes).
    pub fn to_bytes(&self) -> Vec<u8> {
        // 1 Create an empty byte vector.
        let mut bytes = Vec::<u8>::with_capacity(33);

        // 2 Serialize the periodic exemption.
        bytes.extend(self.periodic_credit.to_bytes());

        // 3 Serialize the direct exemption.
        bytes.extend(self.direct_credit.to_le_bytes());

        // 4 Serialize the discount.
        bytes.push(self.discount);

        // 5 Return the bytes.
        bytes
    }

    /// Deserializes the exemption from bytes (`33` bytes current layout, or legacy `40` bytes with u64 PPM discount).
    pub fn from_bytes(bytes: &[u8]) -> Option<Exemption> {
        let (periodic_credit, direct_credit, discount) = match bytes.len() {
            33 => {
                let periodic_credit = PeriodicResource::from_bytes(bytes[0..24].try_into().ok()?)?;
                let direct_credit = u64::from_le_bytes(bytes[24..32].try_into().ok()?);
                let discount = *bytes.get(32)?;
                (periodic_credit, direct_credit, discount)
            }
            40 => {
                // Legacy: discount was PPM out of 1_000_000; map into 0..=200 for /200 semantics.
                let periodic_credit = PeriodicResource::from_bytes(bytes[0..24].try_into().ok()?)?;
                let direct_credit = u64::from_le_bytes(bytes[24..32].try_into().ok()?);
                let discount_ppm = u64::from_le_bytes(bytes[32..40].try_into().ok()?);
                let discount =
                    ((discount_ppm.min(1_000_000) as u128 * 200) / 1_000_000).min(200) as u8;
                (periodic_credit, direct_credit, discount)
            }
            _ => return None,
        };

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
        latest_consumption_timestamp: u64,
        consume_amount: u64,
    ) -> Option<ExemptionSubsidyBreakdown> {
        // 1 Refill and consume the periodic credit.
        let post_periodic_credit_leftover = self.periodic_credit.refill_and_consume(
            current_timestamp,
            latest_consumption_timestamp,
            consume_amount,
        )?;

        // 2 Subsidize the direct credit.
        let direct_credit = self.direct_credit;

        let post_direct_credit_leftover = match direct_credit >= post_periodic_credit_leftover {
            // We can fully subsidize the rest of the direct credit.
            true => {
                let direct_credit_left = direct_credit - post_periodic_credit_leftover;
                self.direct_credit = direct_credit_left;
                0u64
            }
            false => {
                let leftover = post_periodic_credit_leftover - direct_credit;
                self.direct_credit = 0;
                leftover
            }
        };

        // 3 Subsidize the discount: waive `floor(leftover * min(discount, 200) / 200)` (discount is not depleted).
        let leftover_u = post_direct_credit_leftover as u128;
        let discount_factor = (self.discount as u128).min(200);
        let discount_amount = ((leftover_u * discount_factor) / 200) as u64;
        let post_discount_leftover = post_direct_credit_leftover.saturating_sub(discount_amount);

        Some(ExemptionSubsidyBreakdown {
            post_periodic_credit_leftover,
            post_direct_credit_leftover,
            post_discount_leftover,
        })
    }
}
