use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::inscriptive::privileges_manager::elements::exemption::exemption::ExemptionSubsidyBreakdown;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryFees {
    Move {
        base_fee: u64,
        liquidity_fee: u64,
        total_pre_subsidy: u64,
        /// `Some` when a PM exemption row existed and subsidy was applied; `None` when there was no row (full nominal fee).
        subsidy_breakdown: Option<ExemptionSubsidyBreakdown>,
    },
    Liftup {
        base_fee: u64,
        per_lift_fee: u64,
        total_pre_subsidy: u64,
        /// `Some` after a successful subsidy pass; `None` if no PM exemptions (e.g. unregistered liftup) or registered with no exemption row.
        subsidy_breakdown: Option<ExemptionSubsidyBreakdown>,
    },
    Call {
        base_fee: u64,
        total_pre_subsidy: u64,
        subsidy_breakdown: ExemptionSubsidyBreakdown,
    },
    Swapout {
        base_fee: u64,
        total_pre_subsidy: u64,
        /// `Some` when a PM exemption row existed and subsidy was applied; `None` when there was no row (full nominal fee).
        subsidy_breakdown: Option<ExemptionSubsidyBreakdown>,
    },
}

impl EntryFees {
    pub fn json(&self) -> Value {
        let mut obj = Map::new();
        match self {
            EntryFees::Move {
                base_fee,
                liquidity_fee,
                total_pre_subsidy,
                subsidy_breakdown,
            } => {
                obj.insert("entry_kind".to_string(), Value::String("move".to_string()));
                obj.insert("base_fee".to_string(), Value::Number((*base_fee).into()));
                obj.insert(
                    "liquidity_fee".to_string(),
                    Value::Number((*liquidity_fee).into()),
                );
                obj.insert(
                    "total_pre_subsidy".to_string(),
                    Value::Number((*total_pre_subsidy).into()),
                );
                obj.insert(
                    "subsidy_breakdown".to_string(),
                    match subsidy_breakdown {
                        Some(b) => b.json(),
                        None => Value::Null,
                    },
                );
            }
            EntryFees::Liftup {
                base_fee,
                per_lift_fee,
                total_pre_subsidy,
                subsidy_breakdown,
            } => {
                obj.insert("entry_kind".to_string(), Value::String("liftup".to_string()));
                obj.insert("base_fee".to_string(), Value::Number((*base_fee).into()));
                obj.insert("per_lift_fee".to_string(), Value::Number((*per_lift_fee).into()));
                obj.insert(
                    "total_pre_subsidy".to_string(),
                    Value::Number((*total_pre_subsidy).into()),
                );
                obj.insert(
                    "subsidy_breakdown".to_string(),
                    match subsidy_breakdown {
                        Some(b) => b.json(),
                        None => Value::Null,
                    },
                );
            }
            EntryFees::Call {
                base_fee,
                total_pre_subsidy,
                subsidy_breakdown,
            } => {
                obj.insert("entry_kind".to_string(), Value::String("call".to_string()));
                obj.insert("base_fee".to_string(), Value::Number((*base_fee).into()));
                obj.insert(
                    "total_pre_subsidy".to_string(),
                    Value::Number((*total_pre_subsidy).into()),
                );
                obj.insert(
                    "subsidy_breakdown".to_string(),
                    subsidy_breakdown.json(),
                );
            }
            EntryFees::Swapout {
                base_fee,
                total_pre_subsidy,
                subsidy_breakdown,
            } => {
                obj.insert("entry_kind".to_string(), Value::String("swapout".to_string()));
                obj.insert("base_fee".to_string(), Value::Number((*base_fee).into()));
                obj.insert(
                    "total_pre_subsidy".to_string(),
                    Value::Number((*total_pre_subsidy).into()),
                );
                obj.insert(
                    "subsidy_breakdown".to_string(),
                    match subsidy_breakdown {
                        Some(b) => b.json(),
                        None => Value::Null,
                    },
                );
            }
        }

        Value::Object(obj)
    }
}
