use crate::inscriptive::coin_holder::bodies::contract_body::shadow_space::shadow_space::ShadowSpace;
use crate::transmutative::hash::{Hash, HashTag};

/// Satoshi amount.
#[allow(non_camel_case_types)]
type SATOSHI_AMOUNT = u64;

/// A struct for containing BTC balance and shadow space allocations of a contract.
#[derive(Clone)]
pub struct CHContractBody {
    // Contract's BTC balance.
    balance: SATOSHI_AMOUNT,

    // Contract's shadow space.
    shadow_space: ShadowSpace,
}

impl CHContractBody {
    /// Constructs a fresh new contract body.
    pub fn new(balance: SATOSHI_AMOUNT, shadow_space: ShadowSpace) -> Self {
        Self {
            balance: balance,
            shadow_space: shadow_space,
        }
    }

    /// Returns the contract balance.
    pub fn balance(&self) -> SATOSHI_AMOUNT {
        self.balance
    }

    /// Returns the contract shadow space.
    pub fn shadow_space(&self) -> &ShadowSpace {
        &self.shadow_space
    }

    /// Returns a mutable reference to the contract shadow space.
    pub fn shadow_space_mut(&mut self) -> &mut ShadowSpace {
        &mut self.shadow_space
    }

    /// Updates the contract balance.
    pub fn update_balance(&mut self, balance: SATOSHI_AMOUNT) {
        self.balance = balance;
    }

    /// Updates the contract shadow space.
    pub fn update_shadow_space(&mut self, shadow_space: ShadowSpace) {
        self.shadow_space = shadow_space;
    }

    /// Hashes the contract body with the `ContractBody` tag.
    pub fn tagged_hash(&self) -> [u8; 32] {
        let mut preimage: Vec<u8> = Vec::<u8>::new();

        preimage.extend(self.balance.to_le_bytes());
        preimage.extend(self.shadow_space.tagged_hash());

        preimage.hash(Some(HashTag::ContractBody))
    }
}
