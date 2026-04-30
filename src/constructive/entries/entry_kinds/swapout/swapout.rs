use crate::constructive::core_types::target::target::Target;
use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::txout_types::pinless_self::PinlessSelf;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Swapout {
    pub root_account: RootAccount,
    pub amount: u32,
    pub target: Target,
    pub pinless_self: PinlessSelf,
}

impl Swapout {
    pub fn new(
        root_account: RootAccount,
        amount: u32,
        target: Target,
        pinless_self: PinlessSelf,
    ) -> Self {
        Self {
            root_account,
            amount,
            target,
            pinless_self,
        }
    }

    pub fn json(&self) -> Value {
        let mut obj = Map::new();
        obj.insert("entry_kind".to_string(), Value::String("swapout".to_string()));
        obj.insert("account".to_string(), self.root_account.json());
        obj.insert("amount".to_string(), Value::Number(self.amount.into()));
        obj.insert(
            "target".to_string(),
            Value::Number(self.target.targeted_at_batch_height.into()),
        );
        obj.insert("pinless_self".to_string(), self.pinless_self.json());
        Value::Object(obj)
    }

    pub fn serialize(&self) -> Option<Vec<u8>> {
        bincode::serde::encode_to_vec(self, bincode::config::standard()).ok()
    }

    pub fn deserialize(bytes: &[u8]) -> Option<Self> {
        bincode::serde::decode_from_slice::<Self, _>(bytes, bincode::config::standard())
            .ok()
            .map(|(swapout, _)| swapout)
    }
}
