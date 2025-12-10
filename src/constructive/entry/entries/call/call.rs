use crate::constructive::calldata::element::element::CallElement;
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
        account: Account,
        contract: Contract,
        method_index: u8,
        args: Vec<CallElement>,
        ops_budget: u32,
        ops_price_base: u32,
        ops_price_extra_in: Option<u32>,
    ) -> Self {
        Self {
            account,
            contract,
            method_index,
            args,
            ops_budget,
            ops_price_base,
            ops_price_extra_in,
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
        self.account.account_key() == account_key
    }
}
