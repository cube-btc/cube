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
    Config {
        base_fee: u64,
        total_pre_subsidy: u64,
        /// `Some` when a PM exemption row existed and subsidy was applied; `None` when there was no row (full nominal fee).
        subsidy_breakdown: Option<ExemptionSubsidyBreakdown>,
        secondary_aggregation_key_updated: bool,
        projector_config_updated: bool,
        flame_config_updated: bool,
    },
    Deploy {
        base_fee: u64,
        total_pre_subsidy: u64,
        /// `Some` when a PM exemption row existed and subsidy was applied; `None` when there was no row.
        subsidy_breakdown: Option<ExemptionSubsidyBreakdown>,
        initial_balance: u64,
        program_bytes_len: u64,
        contract_id: [u8; 32],
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
            EntryFees::Config {
                base_fee,
                total_pre_subsidy,
                subsidy_breakdown,
                secondary_aggregation_key_updated,
                projector_config_updated,
                flame_config_updated,
            } => {
                obj.insert("entry_kind".to_string(), Value::String("config".to_string()));
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
                obj.insert(
                    "secondary_aggregation_key_updated".to_string(),
                    Value::Bool(*secondary_aggregation_key_updated),
                );
                obj.insert(
                    "projector_config_updated".to_string(),
                    Value::Bool(*projector_config_updated),
                );
                obj.insert(
                    "flame_config_updated".to_string(),
                    Value::Bool(*flame_config_updated),
                );
            }
            EntryFees::Deploy {
                base_fee,
                total_pre_subsidy,
                subsidy_breakdown,
                initial_balance,
                program_bytes_len,
                contract_id,
            } => {
                obj.insert("entry_kind".to_string(), Value::String("deploy".to_string()));
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
                obj.insert(
                    "initial_balance".to_string(),
                    Value::Number((*initial_balance).into()),
                );
                obj.insert(
                    "program_bytes_len".to_string(),
                    Value::Number((*program_bytes_len).into()),
                );
                obj.insert(
                    "contract_id".to_string(),
                    Value::String(hex::encode(contract_id)),
                );
            }
        }

        Value::Object(obj)
    }
}
