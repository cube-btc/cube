use serde::{Deserialize, Serialize};
use crate::inscriptive::privileges_manager::elements::exemption::periodic_resource::periodic_resource::PeriodicResource;

/// A struct for repreferring to the exemption of a resource, transaction fees for the accounts, and occupancy tax for the contracts.
#[derive(Clone, Serialize, Deserialize)]
pub struct Exemption {
    // Primary, periodic, refilling credit.
    pub periodic_credit: PeriodicResource,

    // Secondary, direct, non-refilling credit, applied after the periodic credit, on top of the remaining amount.
    pub direct_credit: u64,

    // Third place, discount, applied after the direct credit, on top of the remaining amount. In PPM (parts per million).
    pub discount: u64,
}

impl Exemption {
    // Constructs a fresh new exemption instance.
    pub fn new(periodic_credit: PeriodicResource, direct_credit: u64, discount: u64) -> Self {
        Self {
            periodic_credit,
            direct_credit,
            discount,
        }
    }

    /// Serializes the exemption to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        // 1 Create an empty byte vector.
        let mut bytes = Vec::<u8>::with_capacity(40);

        // 2 Serialize the periodic exemption.
        bytes.extend(self.periodic_credit.to_bytes());

        // 3 Serialize the direct exemption.
        bytes.extend(self.direct_credit.to_le_bytes());

        // 4 Serialize the discount.
        bytes.extend(self.discount.to_le_bytes());

        // 5 Return the bytes.
        bytes
    }

    /// Deserializes the exemption from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Option<Exemption> {
        // 1 Check if the byte vector is of the expected length.
        if bytes.len() != 40 {
            return None;
        }

        // 2 Deserialize the periodic credit.
        let periodic_credit = PeriodicResource::from_bytes(bytes[0..24].try_into().ok()?)?;

        // 3 Deserialize the direct credit.
        let direct_credit = u64::from_le_bytes(bytes[24..32].try_into().ok()?);

        // 4 Deserialize the discount.
        let discount = u64::from_le_bytes(bytes[32..40].try_into().ok()?);

        // 5 Construct the exemption.
        let exemption = Exemption {
            periodic_credit,
            direct_credit,
            discount: discount,
        };

        // 6 Return the exemption.
        Some(exemption)
    }
}
