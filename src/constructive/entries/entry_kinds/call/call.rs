use crate::constructive::calldata::calldata_elements::calldata_elements::CalldataElement;
use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::entity::contract::contract::Contract;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The holder of a call.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Call {
    /// The account.
    pub account: RootAccount,
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
        account: RootAccount,
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
    pub fn account(&self) -> &RootAccount {
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

    /// Returns the call entry as a JSON object.
    pub fn json(&self) -> Value {
        // 1 Construct the JSON object.
        let mut obj = Map::new();

        // 2 Insert the entry kind.
        obj.insert("entry_kind".to_string(), Value::String("call".to_string()));

        // 3 Insert the account.
        obj.insert("account".to_string(), self.account.json());

        // 4 Insert the contract.
        obj.insert("contract".to_string(), self.contract.json());

        // 5 Insert the method index.
        obj.insert(
            "method_index".to_string(),
            Value::Number((self.method_index as u64).into()),
        );

        // 7 Insert the calldata elements.
        obj.insert(
            "calldata_elements".to_string(),
            serde_json::to_value(&self.calldata_elements)
                .expect("CalldataElement vector must serialize to JSON"),
        );

        // 8 Insert the ops budget.
        obj.insert(
            "ops_budget".to_string(),
            match self.ops_budget {
                Some(n) => Value::Number(n.into()),
                None => Value::Null,
            },
        );

        // 9 Insert the ops price total.
        obj.insert(
            "ops_price_total".to_string(),
            Value::Number(self.ops_price_total().into()),
        );

        // 10 Return the JSON object.
        Value::Object(obj)
    }

    /// Validation from the broader Entry context.
    pub fn entry_validation(&self, account_key: [u8; 32]) -> bool {
        self.account.account_key() == account_key
    }
}
