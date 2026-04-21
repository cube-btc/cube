use crate::constructive::core_types::entities::account::account::account::Account;
use crate::constructive::core_types::entities::account::root_account::root_account::RootAccount;
use crate::constructive::core_types::target::target::Target;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// `Move` is an `Entry` kind for transferring value between accounts.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Move {
    /// The sender root account.
    pub from: RootAccount,

    /// The receiver account.
    pub to: Account,

    /// Amount of coins (satoshis) to move.
    pub amount: u32,

    /// Target execution information.
    pub target: Target,
}

impl Move {
    /// Creates a new `Move` entry kind.
    pub fn new(from: RootAccount, to: Account, amount: u32, target: Target) -> Self {
        Self {
            from,
            to,
            amount,
            target,
        }
    }

    /// Returns the move entry as a JSON object.
    pub fn json(&self) -> Value {
        // 1 Construct the JSON object.
        let mut obj = Map::new();

        // 2 Insert the entry kind.
        obj.insert("entry_kind".to_string(), Value::String("move".to_string()));

        // 3 Insert the sender root account.
        obj.insert("from".to_string(), self.from.json());

        // 4 Insert the receiver account.
        obj.insert("to".to_string(), self.to.json());

        // 5 Insert the amount.
        obj.insert(
            "amount".to_string(),
            Value::Number((self.amount as u64).into()),
        );

        // 6 Insert the target.
        obj.insert(
            "target".to_string(),
            Value::Number(self.target.targeted_at_batch_height.into()),
        );

        // 7 Return the JSON object.
        Value::Object(obj)
    }

    /// Serializes this move entry with bincode.
    pub fn serialize(&self) -> Option<Vec<u8>> {
        bincode::serde::encode_to_vec(self, bincode::config::standard()).ok()
    }

    /// Deserializes a move entry from bincode bytes.
    pub fn deserialize(bytes: &[u8]) -> Option<Self> {
        bincode::serde::decode_from_slice::<Self, _>(bytes, bincode::config::standard())
            .ok()
            .map(|(move_entry, _)| move_entry)
    }
}
