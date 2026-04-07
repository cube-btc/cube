use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::txo::lift::lift::Lift;
use crate::inscriptive::graveyard::graveyard::GRAVEYARD;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::utxo_set::utxo_set::UTXO_SET;
use serde::{Deserialize, Serialize};

/// The `Liftup` struct represents an `Entry` that lifts one or more `Lift` Bitcoin previous transaction outputs.
/// `Liftup` is how BTC is injected into the system from on-chain.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Liftup {
    // The root `Account` that is lifting
    pub account: RootAccount,

    // Spent `Lift` prevtxos (previous transaction outputs)
    pub lift_prevtxos: Vec<Lift>,
}

impl Liftup {
    /// Creates a new Liftup struct.
    pub fn new(account: RootAccount, lift_prevtxos: Vec<Lift>) -> Liftup {
        Self {
            account,
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
