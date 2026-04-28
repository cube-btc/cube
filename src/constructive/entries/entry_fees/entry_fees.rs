use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryFees {
    Move {
        base_fee: u64,
        liquidity_fee: u64,
        total: u64,
    },
    Liftup {
        base_fee: u64,
        per_lift_fee: u64,
        total: u64,
    },
    Call {
        base_fee: u64,
        total: u64,
    },
}

impl EntryFees {
    pub fn json(&self) -> Value {
        let mut obj = Map::new();
        match self {
            EntryFees::Move {
                base_fee,
                liquidity_fee,
                total,
            } => {
                obj.insert("entry_kind".to_string(), Value::String("move".to_string()));
                obj.insert("base_fee".to_string(), Value::Number((*base_fee).into()));
                obj.insert(
                    "liquidity_fee".to_string(),
                    Value::Number((*liquidity_fee).into()),
                );
                obj.insert("total".to_string(), Value::Number((*total).into()));
            }
            EntryFees::Liftup {
                base_fee,
                per_lift_fee,
                total,
            } => {
                obj.insert("entry_kind".to_string(), Value::String("liftup".to_string()));
                obj.insert("base_fee".to_string(), Value::Number((*base_fee).into()));
                obj.insert("per_lift_fee".to_string(), Value::Number((*per_lift_fee).into()));
                obj.insert("total".to_string(), Value::Number((*total).into()));
            }
            EntryFees::Call { base_fee, total } => {
                obj.insert("entry_kind".to_string(), Value::String("call".to_string()));
                obj.insert("base_fee".to_string(), Value::Number((*base_fee).into()));
                obj.insert("total".to_string(), Value::Number((*total).into()));
            }
        }

        Value::Object(obj)
    }
}
