use serde_json::{Map, Value};
use std::collections::HashMap;

/// Account key.
#[allow(non_camel_case_types)]
type ACCOUNT_KEY = [u8; 32];

/// Satoshi amount.
#[allow(non_camel_case_types)]
type SATOSHI_AMOUNT = u64;

/// Sati-satoshi amount.
#[allow(non_camel_case_types)]
type SATI_SATOSHI_AMOUNT = u128;

/// A struct for representing a shadow space of a contract.
#[derive(Clone)]
pub struct ShadowSpace {
    // 1 Total allocated BTC value of the entire shadow space.
    pub allocs_sum: SATOSHI_AMOUNT,

    // 2 Allocated BTC values of each account.
    pub allocs: HashMap<ACCOUNT_KEY, SATI_SATOSHI_AMOUNT>,
}

impl ShadowSpace {
    /// Constructs a fresh new shadow space.
    pub fn fresh_new() -> Self {
        Self {
            allocs_sum: 0,
            allocs: HashMap::new(),
        }
    }
    /// Constructs a fresh new shadow space.
    pub fn new(
        allocs_sum: SATOSHI_AMOUNT,
        allocs: HashMap<ACCOUNT_KEY, SATI_SATOSHI_AMOUNT>,
    ) -> Self {
        // 1 Construct the shadow space.
        let shadow_space = Self {
            allocs_sum: allocs_sum,
            allocs: allocs,
        };

        // 2 Return the shadow space.
        shadow_space
    }

    /// Updates the allocations sum.
    pub fn update_allocs_sum(&mut self, new_value: SATOSHI_AMOUNT) {
        // 1 Update the allocations sum.
        self.allocs_sum = new_value;
    }

    /// Inserts (or updates) an allocation into the shadow space.
    pub fn insert_update_alloc(
        &mut self,
        account_key: ACCOUNT_KEY,
        alloc_value: SATI_SATOSHI_AMOUNT,
    ) {
        // 1 Insert the allocation into the allocations map.
        self.allocs.insert(account_key, alloc_value);
    }

    /// Removes an allocation from the shadow space.
    pub fn remove_alloc(&mut self, account_key: ACCOUNT_KEY) -> bool {
        // 1 Remove the allocation from the allocations map.
        match self.allocs.remove(&account_key) {
            Some(_) => true,
            None => false,
        }
    }

    /// Returns the shadow space as a JSON object.
    pub fn json(&self) -> Value {
        // 1 Construct the shadow space JSON object.
        let mut obj = Map::new();

        // 2 Insert the allocs sum.
        obj.insert(
            "allocs_sum".to_string(),
            Value::String(self.allocs_sum.to_string()),
        );

        // 3 Insert the allocations.
        obj.insert(
            "allocs".to_string(),
            Value::Object(
                self.allocs
                    .iter()
                    .map(|(account_key, alloc_value)| {
                        (
                            hex::encode(account_key),
                            Value::String(alloc_value.to_string()),
                        )
                    })
                    .collect(),
            ),
        );

        // 4 Return the JSON object.
        Value::Object(obj)
    }
}
