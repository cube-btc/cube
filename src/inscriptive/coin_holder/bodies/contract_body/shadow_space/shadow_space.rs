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
    // Total allocated BTC value of the entire shadow space.
    allocs_sum: SATOSHI_AMOUNT,

    // Allocated BTC values of each account.
    allocs: HashMap<ACCOUNT_KEY, SATI_SATOSHI_AMOUNT>,
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
        // Return the shadow space.
        let shadow_space = Self {
            allocs_sum: allocs_sum,
            allocs: allocs,
        };

        // Return the shadow space.
        shadow_space
    }

    /// Returns the allocations sum.
    pub fn allocs_sum(&self) -> SATOSHI_AMOUNT {
        self.allocs_sum
    }

    /// Returns the number of allocations.
    pub fn allocs_len(&self) -> usize {
        self.allocs.len()
    }

    /// Returns a clone of the allocations map.
    pub fn allocs(&self) -> &HashMap<ACCOUNT_KEY, SATI_SATOSHI_AMOUNT> {
        &self.allocs
    }

    /// Updates the allocations sum.
    pub fn update_allocs_sum(&mut self, new_value: SATOSHI_AMOUNT) {
        // Update the allocations sum.
        self.allocs_sum = new_value;
    }

    /// Inserts an allocation into the shadow space.
    pub fn insert_alloc(
        &mut self,
        account_key: ACCOUNT_KEY,
        alloc_value: SATI_SATOSHI_AMOUNT,
    ) -> bool {
        // Insert the allocation into the allocations map.
        match self.allocs.insert(account_key, alloc_value) {
            Some(_) => false,
            None => true,
        }
    }

    /// Removes an allocation from the shadow space.
    pub fn remove_alloc(&mut self, account_key: ACCOUNT_KEY) -> bool {
        // Remove the allocation from the allocations map.
        match self.allocs.remove(&account_key) {
            Some(_) => true,
            None => false,
        }
    }
}
