use crate::constructive::calldata::element::element::CalldataElement;
use crate::constructive::entity::account::account::Account;
use crate::constructive::entity::contract::contract::Contract;
use serde::{Deserialize, Serialize};

/// The holder of a call.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Call {
    /// The account.
    pub account: Account,
    /// The contract id.
    pub contract: Contract,
    /// The method index.
    pub method_index: u8,
    /// The arguments.
    pub calldata_elements: Vec<CalldataElement>,
    /// The ops budget.
    pub ops_budget: Option<u32>,
    /// The base ops price.
    pub ops_price_base: u32,
    /// The extra ops price.
    pub ops_price_overhead: Option<u32>,
}

impl Call {
    /// Creates a new call holder.
    pub fn new(
        account: Account,
        contract: Contract,
        method_index: u8,
        calldata_elements: Vec<CalldataElement>,
        ops_budget: Option<u32>,
        ops_price_base: u32,
        ops_price_overhead: Option<u32>,
    ) -> Self {
        Self {
            account,
            contract,
            method_index,
            calldata_elements,
            ops_budget,
            ops_price_base,
            ops_price_overhead,
        }
    }

    /// Returns the `Account`.
    pub fn account(&self) -> &Account {
        &self.account
    }

    /// Returns the `Contract`.
    pub fn contract(&self) -> &Contract {
        &self.contract
    }

    /// Returns the method index.
    pub fn method_index(&self) -> u8 {
        self.method_index
    }

    /// Returns the arguments.
    pub fn calldata_elements(&self) -> Vec<CalldataElement> {
        self.calldata_elements.clone()
    }

    /// Returns the ops budget.
    pub fn ops_budget(&self) -> Option<u32> {
        self.ops_budget
    }

    /// Returns the base ops price.
    pub fn ops_price_base(&self) -> u32 {
        self.ops_price_base
    }

    /// Returns the extra ops price.
    pub fn ops_price_overhead(&self) -> Option<u32> {
        self.ops_price_overhead
    }

    /// Returns the total ops price.
    pub fn ops_price_total(&self) -> u32 {
        self.ops_price_base + self.ops_price_overhead.unwrap_or(0)
    }

    /// Validation from the broader Entry context.
    pub fn entry_validation(&self, account_key: [u8; 32]) -> bool {
        self.account.account_key() == account_key
    }
}
