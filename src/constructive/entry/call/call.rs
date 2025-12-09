use crate::constructive::calldata::element::element::CallElement;
use serde::{Deserialize, Serialize};

/// The holder of a call.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Call {
    /// The account key.
    pub account_key: [u8; 32],
    /// The contract id.
    pub contract_id: [u8; 32],
    /// The method index.
    pub method_index: u8,
    /// The arguments.
    pub args: Vec<CallElement>,
    /// The ops budget.
    pub ops_budget: u32,
    /// The base ops price.
    pub ops_price_base: u32,
    /// The extra ops price.
    pub ops_price_extra_in: Option<u32>,
}

impl Call {
    /// Creates a new call holder.
    pub fn new(
        account_key: [u8; 32],
        contract_id: [u8; 32],
        method_index: u8,
        args: Vec<CallElement>,
        ops_budget: u32,
        ops_price_base: u32,
        ops_price_extra_in: Option<u32>,
    ) -> Self {
        Self {
            account_key,
            contract_id,
            method_index,
            args,
            ops_budget,
            ops_price_base,
            ops_price_extra_in,
        }
    }

    /// Returns the account key.
    pub fn account_key(&self) -> [u8; 32] {
        self.account_key
    }

    /// Returns the contract id.
    pub fn contract_id(&self) -> [u8; 32] {
        self.contract_id
    }

    /// Returns the method index.
    pub fn method_index(&self) -> u8 {
        self.method_index
    }

    /// Returns the arguments.
    pub fn args(&self) -> Vec<CallElement> {
        self.args.clone()
    }

    /// Returns the ops budget.
    pub fn ops_budget(&self) -> u32 {
        self.ops_budget
    }

    /// Returns the base ops price.
    pub fn ops_price_base(&self) -> u32 {
        self.ops_price_base
    }

    /// Returns the extra ops price.
    pub fn ops_price_extra_in(&self) -> Option<u32> {
        self.ops_price_extra_in
    }

    /// Returns the total ops price.
    pub fn ops_price_total(&self) -> u32 {
        self.ops_price_base + self.ops_price_extra_in.unwrap_or(0)
    }

    /// Validation from the broader Entry context.
    pub fn entry_validation(&self, account_key: [u8; 32]) -> bool {
        self.account_key == account_key
    }
}
