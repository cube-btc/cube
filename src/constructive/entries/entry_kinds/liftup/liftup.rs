use crate::constructive::core_types::target::target::Target;
use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::txo::lift::lift::Lift;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The `Liftup` struct represents an `Entry` that lifts one or more `Lift` Bitcoin previous transaction outputs.
/// `Liftup` is how BTC is injected into the system from on-chain.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Liftup {
    // The `RootAccount` that is lifting
    pub root_account: RootAccount,

    // The `Target` of the `Liftup`.
    pub target: Target,

    // Spent `Lift` prevtxos (previous transaction outputs)
    pub lift_tx_inputs: Vec<Lift>,
}

impl Liftup {
    /// Creates a new Liftup struct.
    pub fn new(root_account: RootAccount, target: Target, lift_tx_inputs: Vec<Lift>) -> Liftup {
        Self {
            root_account,
            target,
            lift_tx_inputs,
        }
    }

    /// Returns the total liftup sum value in satoshis.
    pub fn liftup_sum_value_in_satoshis(&self) -> u64 {
        self.lift_tx_inputs
            .iter()
            .map(|lift| lift.lift_value_in_satoshis())
            .sum()
    }

    /// Returns the liftup entry as a JSON object.
    pub fn json(&self) -> Value {
        // 1 Construct the JSON object.
        let mut obj = Map::new();

        // 2 Insert the kind.
        obj.insert("kind".to_string(), Value::String("liftup".to_string()));

        // 3 Insert the root account.
        obj.insert("root_account".to_string(), self.root_account.json());

        // 4 Insert the target.
        obj.insert(
            "target".to_string(),
            Value::Number(self.target.targeted_at_batch_height.into()),
        );

        // 5 Insert the lift tx inputs.
        obj.insert(
            "lift_tx_inputs".to_string(),
            Value::Array(self.lift_tx_inputs.iter().map(|lift| lift.json()).collect()),
        );

        // 6 Return the JSON object.
        Value::Object(obj)
    }
}
