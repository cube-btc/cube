/// Satoshi amount.
#[allow(non_camel_case_types)]
type SATOSHI_AMOUNT = u64;

/// Sati-satoshi amount.
#[allow(non_camel_case_types)]
type SATI_SATOSHI_AMOUNT = u128;

/// A struct for containing account balance and shadow allocs sum of all contracts.
#[derive(Clone)]
pub struct CHAccountBody {
    // Account balance.
    pub balance: SATOSHI_AMOUNT,

    // Individual shadow allocs sum of all contracts.
    pub shadow_allocs_sum: SATI_SATOSHI_AMOUNT,
}
