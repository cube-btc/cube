use crate::constructive::core_types::entities::account::root_account::root_account::RootAccount;
use crate::constructive::core_types::target::target::Target;
use crate::executive::executable::executable::Program;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// `Deploy` is an `Entry` kind for deploying a program.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deploy {
    /// The root account deploying the program.
    pub root_account: RootAccount,
    /// The program to deploy.
    pub program: Program,
    /// Initial balance allocated on deployment.
    pub initial_balance: u32,
    /// Target execution information.
    pub target: Target,
}

impl Deploy {
    /// Creates a new `Deploy` entry kind.
    pub fn new(root_account: RootAccount, program: Program, initial_balance: u32, target: Target) -> Self {
        Self {
            root_account,
            program,
            initial_balance,
            target,
        }
    }

    /// Returns the deploy entry as a JSON object.
    pub fn json(&self) -> Value {
        let mut obj = Map::new();
        obj.insert("entry_kind".to_string(), Value::String("deploy".to_string()));
        obj.insert("root_account".to_string(), self.root_account.json());
        obj.insert("program".to_string(), self.program.json());
        obj.insert(
            "initial_balance".to_string(),
            Value::Number((self.initial_balance as u64).into()),
        );
        obj.insert(
            "target".to_string(),
            Value::Number(self.target.targeted_at_batch_height.into()),
        );
        Value::Object(obj)
    }
}
