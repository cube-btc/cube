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
    pub allocs_sum: SATOSHI_AMOUNT,

    // Allocated BTC values of each account.
    pub allocs: HashMap<ACCOUNT_KEY, SATI_SATOSHI_AMOUNT>,
}
