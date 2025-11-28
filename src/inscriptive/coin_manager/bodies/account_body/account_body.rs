use serde_json::{Map, Value};

/// Satoshi amount.
#[allow(non_camel_case_types)]
type SatoshiAmount = u64;

/// Sati-satoshi amount.
#[allow(non_camel_case_types)]
type SatiSatoshiAmount = u128;

/// A struct for containing BTC balance and shadow allocs sum of an account.
#[derive(Clone)]
pub struct CMAccountBody {
    // Account's BTC balance.
    pub balance: SatoshiAmount,

    // Account's shadow allocs sum.
    pub shadow_allocs_sum: SatiSatoshiAmount,
}

impl CMAccountBody {
    /// Constructs a fresh new account body.
    pub fn new(balance: SatoshiAmount, shadow_allocs_sum: SatiSatoshiAmount) -> Self {
        Self {
            balance: balance,
            shadow_allocs_sum: shadow_allocs_sum,
        }
    }

    /// Updates the account balance.
    pub fn update_balance(&mut self, balance: SatoshiAmount) {
        self.balance = balance;
    }

    /// Updates the account shadow allocs sum.
    pub fn update_shadow_allocs_sum(&mut self, shadow_allocs_sum: SatiSatoshiAmount) {
        self.shadow_allocs_sum = shadow_allocs_sum;
    }

    /// Returns the account body as a JSON object.
    pub fn json(&self) -> Value {
        // 1 Construct the account body JSON object.
        let mut obj = Map::new();

        // 2 Insert the balance.
        obj.insert(
            "balance".to_string(),
            Value::String(self.balance.to_string()),
        );

        // 3 Insert the shadow allocs sum.
        obj.insert(
            "shadow_allocs_sum".to_string(),
            Value::String(self.shadow_allocs_sum.to_string()),
        );

        // 4 Return the JSON object.
        Value::Object(obj)
    }
}
