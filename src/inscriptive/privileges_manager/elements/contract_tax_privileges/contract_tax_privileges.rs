use crate::inscriptive::privileges_manager::elements::periodic_resource::periodic_resource::PeriodicResource;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ContractTaxPrivileges {
    pub periodic_credit: PeriodicResource,

    pub direct_credit: u64,

    pub discount: u64,
}

impl ContractTaxPrivileges {
    // Constructs a fresh new contract tax privileges instance.
    pub fn new(periodic_credit: PeriodicResource, direct_credit: u64, discount: u64) -> Self {
        Self {
            periodic_credit,
            direct_credit,
            discount,
        }
    }

    /// Serializes the contract tax privileges to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        // 1 Create an empty byte vector.
        let mut bytes = Vec::<u8>::with_capacity(40);

        // 2 Serialize the periodic credit.
        bytes.extend(self.periodic_credit.to_bytes());

        // 3 Serialize the direct credit.
        bytes.extend(self.direct_credit.to_le_bytes());

        // 4 Serialize the discount.
        bytes.extend(self.discount.to_le_bytes());

        // 5 Return the bytes.
        bytes
    }

    /// Deserializes the contract tax privileges from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Option<ContractTaxPrivileges> {
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

        // 5 Construct the account tx fee privileges.
        let contract_tax_privileges = ContractTaxPrivileges {
            periodic_credit,
            direct_credit,
            discount: discount,
        };

        // 6 Return the account tx fee privileges.
        Some(contract_tax_privileges)
    }
}
