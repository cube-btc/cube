use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::txo::lift::lift::Lift;
use crate::inscriptive::registery_manager::registery_manager::REGISTERY_MANAGER;
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

    /// Checks whether the `Liftup` is indeed a valid liftup.
    pub async fn validate(
        &self,
        registery_manager: &REGISTERY_MANAGER,
        utxo_set: &UTXO_SET,
    ) -> bool {
        // 1 Validate the account.
        if !self.account.validate(registery_manager).await {
            return false;
        }

        // 2 Validate the `Lift`s in the `Liftup`.
        {
            let _utxo_set = utxo_set.lock().await;

            if !_utxo_set.validate_lifts(&self.lift_prevtxos) {
                return false;
            }
        }

        // 3 Return true.
        true
    }
}
