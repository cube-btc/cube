use crate::inscriptive::coin_manager::bodies::contract_body::shadow_space::shadow_space::ShadowSpace;
use serde_json::{Map, Value};

/// Satoshi amount.
#[allow(non_camel_case_types)]
type SatoshiAmount = u64;

/// A struct for containing BTC balance and shadow space of a contract.
#[derive(Clone)]
pub struct CMContractBody {
    // Contract's BTC balance.
    pub balance: SatoshiAmount,

    // Contract's shadow space.
    pub shadow_space: ShadowSpace,
}

impl CMContractBody {
    /// Constructs a fresh new contract body.
    pub fn new(balance: SatoshiAmount, shadow_space: ShadowSpace) -> Self {
        Self {
            balance: balance,
            shadow_space: shadow_space,
        }
    }

    /// Updates the contract balance.
    pub fn update_balance(&mut self, balance: SatoshiAmount) {
        self.balance = balance;
    }

    /// Updates the contract shadow space.
    pub fn update_shadow_space(&mut self, shadow_space: ShadowSpace) {
        self.shadow_space = shadow_space;
    }

    /// Returns the contract body as a JSON object.
    pub fn json(&self) -> Value {
        // 1 Construct the contract body JSON object.
        let mut obj = Map::new();

        // 2 Insert the balance.
        obj.insert(
            "balance".to_string(),
            Value::String(self.balance.to_string()),
        );

        // 3 Insert the shadow space.
        obj.insert("shadow_space".to_string(), self.shadow_space.json());

        // 4 Return the JSON object.
        Value::Object(obj)
    }
}
