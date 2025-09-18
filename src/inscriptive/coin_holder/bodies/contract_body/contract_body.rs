use crate::inscriptive::coin_holder::bodies::contract_body::shadow_space::shadow_space::ShadowSpace;

/// Satoshi amount.
#[allow(non_camel_case_types)]
type SATOSHI_AMOUNT = u64;

/// A struct for containing BTC balance and shadow space allocations of a contract.
#[derive(Clone)]
pub struct CHContractBody {
    // Contract's BTC balance.
    pub balance: SATOSHI_AMOUNT,

    // Contract's shadow space.
    pub shadow_space: ShadowSpace,
}
