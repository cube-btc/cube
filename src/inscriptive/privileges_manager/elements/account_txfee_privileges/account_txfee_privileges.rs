use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct AccountTxFeePrivileges {
    // PERIODIC CREDIT (PRIMARY).
    // Represents the VIP card limit (in satoshis).
    // Akin to a credit card limit that can be spent periodically.
    // The limit is the maximum amount that can be spent in a given period.
    // The left is the amount left from the latest transaction.
    // The period is the duration of the credit period.
    pub periodic_credit_limit: u64,
    pub periodic_credit_left: u64,
    pub periodic_credit_period: u64,

    // DIRECT CREDIT (SECONDARY).
    // This is like a gift card balance that can be spent directly, but preceed  by the periodic credit.
    // If there is an overflow from the periodic credit, this will be used to cover the difference (if any).
    pub direct_credit: u64,

    // SPEND DISCOUNT (SECONDARY).
    // Represents the VIP card discount (in PPM - parts per million).
    // If there is an overflow from the periodic credit, this will be used to apply the discount to the difference (if set).
    pub discount: u64,
}

impl AccountTxFeePrivileges {
    // Constructs a fresh new account tx fee privileges instance.
    pub fn new(
        periodic_credit_limit: u64,
        periodic_credit_left: u64,
        periodic_credit_period: u64,
        direct_credit: u64,
        discount: u64,
    ) -> Self {
        Self {
            periodic_credit_limit,
            periodic_credit_left,
            periodic_credit_period,
            direct_credit,
            discount,
        }
    }

    /// Serializes the account tx fee privileges to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        // 1 Create an empty byte vector.
        let mut bytes = Vec::<u8>::with_capacity(40);

        // 2 Serialize the periodic credit limit.
        bytes.extend(self.periodic_credit_limit.to_le_bytes());

        // 3 Serialize the periodic credit left.
        bytes.extend(self.periodic_credit_left.to_le_bytes());

        // 4 Serialize the periodic credit period.
        bytes.extend(self.periodic_credit_period.to_le_bytes());

        // 5 Serialize the direct credit.
        bytes.extend(self.direct_credit.to_le_bytes());

        // 6 Serialize the discount.
        bytes.extend(self.discount.to_le_bytes());

        // 7 Return the bytes.
        bytes
    }

    /// Deserializes the account tx fee privileges from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Option<AccountTxFeePrivileges> {
        // We expect exactly 40 bytes (5 * 8) for the account tx fee privileges.
        if bytes.len() != 40 {
            return None;
        }

        // 1 Deserialize the periodic credit limit.
        let periodic_credit_limit = u64::from_le_bytes(bytes[0..8].try_into().ok()?);

        // 2 Deserialize the periodic credit left.
        let periodic_credit_left = u64::from_le_bytes(bytes[8..16].try_into().ok()?);

        // 3 Deserialize the periodic credit period.
        let periodic_credit_period = u64::from_le_bytes(bytes[16..24].try_into().ok()?);

        // 4 Deserialize the direct credit.
        let direct_credit = u64::from_le_bytes(bytes[24..32].try_into().ok()?);

        // 5 Deserialize the discount.
        let discount = u64::from_le_bytes(bytes[32..40].try_into().ok()?);

        // 6 Construct the account tx fee privileges.
        let account_txfee_privileges = Self {
            periodic_credit_limit,
            periodic_credit_left,
            periodic_credit_period,
            direct_credit,
            discount: discount,
        };

        // 7 Return the account tx fee privileges.
        Some(account_txfee_privileges)
    }
}
