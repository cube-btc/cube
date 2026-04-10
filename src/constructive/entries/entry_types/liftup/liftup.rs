use crate::constructive::core_types::target::target::Target;
use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::txo::lift::lift::Lift;
use serde::{Deserialize, Serialize};

/// The `Liftup` struct represents an `Entry` that lifts one or more `Lift` Bitcoin previous transaction outputs.
/// `Liftup` is how BTC is injected into the system from on-chain.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Liftup {
    // The `RootAccount` that is lifting
    pub root_account: RootAccount,

    // The `Target` of the `Liftup`.
    pub target: Target,

    // Spent `Lift` prevtxos (previous transaction outputs)
    pub lift_prevtxos: Vec<Lift>,
}

impl Liftup {
    /// Creates a new Liftup struct.
    pub fn new(root_account: RootAccount, target: Target, lift_prevtxos: Vec<Lift>) -> Liftup {
        Self {
            root_account,
            target,
            lift_prevtxos,
        }
    }

    /// Returns the total liftup sum value in satoshis.
    pub fn liftup_sum_value_in_satoshis(&self) -> u64 {
        self.lift_prevtxos
            .iter()
            .map(|lift| lift.txout().value.to_sat())
            .sum()
    }
    
}
