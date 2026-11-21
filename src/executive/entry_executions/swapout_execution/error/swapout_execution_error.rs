use crate::inscriptive::coin_manager::errors::balance_update_errors::CMAccountBalanceDownError;
use crate::constructive::entity::account::root_account::registered_and_configured_root_account::ext::sync_with_registry::sync_with_registry_error::RegisteredAndConfiguredRootAccountSyncWithRegistryError;
use crate::constructive::entity::account::root_account::registered_but_unconfigured_root_account::ext::sync_with_registry::sync_with_registry_error::RegisteredButUnconfiguredRootAccountSyncWithRegistryError;

#[derive(Debug, Clone)]
pub enum SwapoutExecutionError {
    PinlessSelfDefaultFailedToGetCalculatedScriptpubkeyError,
    PinlessSelfDefaultScriptpubkeyMismatchError,
    SwapoutAmountBelowDustMin {
        amount: u32,
        dust_min: u32,
    },
    UnexpectedUnregisteredRootAccountError,
    RegisteredButUnconfiguredRootAccountValidateBLSKeyError,
    RegisteredButUnconfiguredRootAccountInvalidAuthorizationSignatureError,
    RegisteredButUnconfiguredRootAccountSyncWithRegistryError(
        RegisteredButUnconfiguredRootAccountSyncWithRegistryError,
    ),
    RegisteredAndConfiguredRootAccountSyncWithRegistryError(
        RegisteredAndConfiguredRootAccountSyncWithRegistryError,
    ),
    CoinManagerAccountBalanceDownError(CMAccountBalanceDownError),
    AmountPlusFeesOverflow,

    /// Root account has no liveness flag in the privileges manager.
    RootAccountLivenessFlagNotFound([u8; 32]),
    /// Root account's ToBeFrozen takes-effect timestamp has elapsed.
    RootAccountFrozen([u8; 32]),
    /// Root account's ToBeDestroyed takes-effect timestamp has elapsed.
    RootAccountDestroyed([u8; 32]),

    FailedToApplyFeesSubsidy,
}
