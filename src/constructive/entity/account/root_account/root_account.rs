use crate::constructive::entity::account::root_account::registered_and_bls_configured_root_account::registered_and_bls_configured_root_account::RegisteredAndBLSConfiguredRootAccount;
use crate::constructive::entity::account::root_account::registered_but_bls_unconfigured_root_account::registered_but_bls_unconfigured_root_account::RegisteredButBLSUnconfiguredRootAccount;
use crate::constructive::entity::account::root_account::unregistered_root_account::unregistered_root_account::UnregisteredRootAccount;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum RootAccount {
    // A fresh, unregistered (thus unranked), and unconfigured account.
    UnregisteredRootAccount(UnregisteredRootAccount),

    // A registered account with a BLS key unconfigured.
    RegisteredButBLSUnconfiguredRootAccount(RegisteredButBLSUnconfiguredRootAccount),

    // A registered account with a configured BLS key.
    RegisteredAndBLSConfiguredRootAccount(RegisteredAndBLSConfiguredRootAccount),
}
