use crate::constructive::entity::account::root_account::registered_and_configured_root_account::ext::sync_with_registry::sync_with_registry_error::RegisteredAndConfiguredRootAccountSyncWithRegistryError;
use crate::constructive::entity::account::root_account::registered_but_unconfigured_root_account::ext::sync_with_registry::sync_with_registry_error::RegisteredButUnconfiguredRootAccountSyncWithRegistryError;
use crate::inscriptive::coin_manager::errors::balance_update_errors::CMAccountBalanceDownError;
use crate::inscriptive::registry::errors::update_account_flame_config_error::RMUpdateAccountFlameConfigError;
use crate::inscriptive::registry::errors::update_account_projector_config_error::RMUpdateAccountProjectorConfigError;
use crate::inscriptive::registry::errors::update_account_secondary_aggregation_key_error::RMUpdateAccountSecondaryAggregationKeyError;

/// Errors associated with executing a `Config` entry.
#[derive(Debug, Clone)]
pub enum ConfigExecutionError {
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
    RegistrySetOrUpdateSecondaryAggregationKeyError(RMUpdateAccountSecondaryAggregationKeyError),
    RegistrySetOrUpdateProjectorConfigError(RMUpdateAccountProjectorConfigError),
    RegistrySetOrUpdateFlameConfigError(RMUpdateAccountFlameConfigError),
    FailedToApplyFeesSubsidy,
    ConfigByteFeeOverflow,
    ConfigTotalPreSubsidyOverflow,
    ConfigFeeDebitOverflow,
}
