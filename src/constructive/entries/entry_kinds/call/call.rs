use crate::constructive::calldata::calldata_elements::calldata_element::CalldataElement;
use crate::constructive::core_types::method_index::method_index::MethodIndex;
use crate::constructive::core_types::ops_budget::ops_budget::OpsBudget;
use crate::constructive::core_types::ops_price::ops_price::OpsPrice;
use crate::constructive::core_types::target::target::Target;
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
    pub method_index: MethodIndex,

    /// The arguments.
    pub calldata_elements: Vec<CalldataElement>,

    /// The ops budget.
    pub ops_budget: OpsBudget,

    /// The ops price.
    pub ops_price: OpsPrice,

    /// The target.
    pub target: Target,
}

impl Call {
    /// Creates a new call holder.
    pub fn new(
        account: RootAccount,
        contract: Contract,
        method_index: MethodIndex,
        calldata_elements: Vec<CalldataElement>,
        ops_budget: OpsBudget,
        ops_price: OpsPrice,
        target: Target,
    ) -> Self {
        Self {
            account,
            contract,
            method_index,
            calldata_elements,
            ops_budget,
            ops_price,
            target,
        }
    }

    /// Returns the `Account`.
    pub fn account(&self) -> &RootAccount {
        &self.account
    }

    /// Returns the contract.
    pub fn contract(&self) -> &Contract {
        &self.contract
    }

    /// Returns the method index.
    pub fn method_index(&self) -> u16 {
        self.method_index.index()
    }

    /// Returns the arguments.
    pub fn calldata_elements(&self) -> Vec<CalldataElement> {
        self.calldata_elements.clone()
    }

    /// Returns the ops budget, if set.
    pub fn ops_budget(&self) -> Option<u32> {
        self.ops_budget.ops_budget
    }

    /// Returns the ops price.
    pub fn ops_price(&self) -> &OpsPrice {
        &self.ops_price
    }

    /// Returns the ops price in ppm.
    pub fn ops_price_ppm(&self) -> u64 {
        self.ops_price.ops_price_ppm
    }

    /// Returns the ops price used at execution time (ppm as `u32`).
    pub fn ops_price_total(&self) -> u32 {
        self.ops_price.ops_price_ppm as u32
    }

    /// Returns the call entry as a JSON object.
    pub fn json(&self) -> Value {
        let mut obj = Map::new();

        obj.insert("entry_kind".to_string(), Value::String("call".to_string()));
        obj.insert("root_account".to_string(), self.account.json());
        obj.insert("contract".to_string(), self.contract.json());
        obj.insert(
            "method_index".to_string(),
            Value::Number((self.method_index.index() as u64).into()),
        );
        obj.insert(
            "calldata_elements".to_string(),
            serde_json::to_value(&self.calldata_elements)
                .expect("CalldataElement vector must serialize to JSON"),
        );
        obj.insert(
            "ops_budget".to_string(),
            match self.ops_budget.ops_budget {
                Some(n) => Value::Number(n.into()),
                None => Value::Null,
            },
        );
        obj.insert(
            "ops_price_ppm".to_string(),
            Value::Number(self.ops_price.ops_price_ppm.into()),
        );
        obj.insert(
            "target".to_string(),
            Value::Number(self.target.targeted_at_batch_height.into()),
        );

        Value::Object(obj)
    }

    /// Validation from the broader Entry context.
    pub fn entry_validation(&self, account_key: [u8; 32]) -> bool {
        self.account.account_key() == account_key
    }

    /// Serializes this call with bincode (same config as wire payloads elsewhere).
    pub fn serialize(&self) -> Option<Vec<u8>> {
        bincode::serde::encode_to_vec(self, bincode::config::standard()).ok()
    }

    /// Deserializes a call from bincode bytes.
    pub fn deserialize(bytes: &[u8]) -> Option<Self> {
        bincode::serde::decode_from_slice::<Self, _>(bytes, bincode::config::standard())
            .ok()
            .map(|(call, _)| call)
    }
}
