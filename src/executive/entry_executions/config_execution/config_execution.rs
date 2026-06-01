use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::entry::entry_fees::entry_fees::EntryFees;
use crate::constructive::entry::entry_kinds::config::config::Config;
use crate::executive::entry_executions::config_execution::error::config_execution_error::ConfigExecutionError;
use crate::executive::exec_ctx::exec_ctx::ExecCtx;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::coin_manager::errors::balance_update_errors::CMAccountBalanceDownError;
use crate::inscriptive::privileges_manager::elements::exemption::exemption::ExemptionSubsidyBreakdown;
use crate::inscriptive::privileges_manager::errors::update_error::PMUpdateAccountError;

impl ExecCtx {
    /// Executes a `Config` entry.
    pub async fn execute_config_internal(
        &mut self,
        config: &Config,
        execution_timestamp: u64,
    ) -> Result<EntryFees, ConfigExecutionError> {
        let account_key = config.root_account.account_key();
        let params_holder = {
            let _params_manager = self._params_manager.lock().unwrap();
            _params_manager.get_params_holder()
        };
        let base_fee = params_holder.config_entry_base_fee;
        let per_config_byte_fee = params_holder.config_entry_per_config_byte_fee;
        let config_bytes_len = config
            .secondary_aggregation_key
            .as_ref()
            .map(|v| v.len() as u64)
            .unwrap_or(0)
            + if config.projector_config.is_some() { 32 } else { 0 }
            + config
                .flame_config
                .as_ref()
                .map(|cfg| cfg.to_bytes().len() as u64)
                .unwrap_or(0);
        let config_byte_fee = config_bytes_len
            .checked_mul(per_config_byte_fee)
            .ok_or(ConfigExecutionError::ConfigByteFeeOverflow)?;
        let fees_pre_subsidy = base_fee
            .checked_add(config_byte_fee)
            .ok_or(ConfigExecutionError::ConfigTotalPreSubsidyOverflow)?;

        let latest_activity_timestamp = {
            let _registry = self.registry.lock().await;
            _registry
                .get_account_last_activity_timestamp(account_key)
                .unwrap_or(0)
        };

        let (fees_after_subsidy, subsidy_breakdown) = match &config.root_account {
            RootAccount::UnregisteredRootAccount(_) => {
                return Err(ConfigExecutionError::UnexpectedUnregisteredRootAccountError);
            }
            RootAccount::RegisteredButUnconfiguredRootAccount(
                registered_but_unconfigured_root_account,
            ) => {
                if !registered_but_unconfigured_root_account.validate_bls_key() {
                    return Err(
                        ConfigExecutionError::RegisteredButUnconfiguredRootAccountValidateBLSKeyError,
                    );
                }

                if !registered_but_unconfigured_root_account.verify_authorization_signature() {
                    return Err(
                        ConfigExecutionError::RegisteredButUnconfiguredRootAccountInvalidAuthorizationSignatureError,
                    );
                }

                registered_but_unconfigured_root_account
                    .sync_with_registry(execution_timestamp, &self.registry)
                    .await
                    .map_err(
                        ConfigExecutionError::RegisteredButUnconfiguredRootAccountSyncWithRegistryError,
                    )?;
                self.apply_subsidy_config(
                    account_key,
                    execution_timestamp,
                    fees_pre_subsidy,
                    latest_activity_timestamp,
                )
                .await?
            }
            RootAccount::RegisteredAndConfiguredRootAccount(
                registered_and_configured_root_account,
            ) => {
                registered_and_configured_root_account
                    .sync_with_registry(execution_timestamp, &self.registry)
                    .await
                    .map_err(
                        ConfigExecutionError::RegisteredAndConfiguredRootAccountSyncWithRegistryError,
                    )?;
                self.apply_subsidy_config(
                    account_key,
                    execution_timestamp,
                    fees_pre_subsidy,
                    latest_activity_timestamp,
                )
                .await?
            }
        };

        let fee_debit = 0u64
            .checked_add(fees_after_subsidy)
            .ok_or(ConfigExecutionError::ConfigFeeDebitOverflow)?;
        
        decrease_account_balance_with_coin_manager(&self.coin_manager, account_key, fee_debit)
            .await
            .map_err(ConfigExecutionError::CoinManagerAccountBalanceDownError)?;

        {
            let mut registry = self.registry.lock().await;

            if let Some(secondary_aggregation_key) = &config.secondary_aggregation_key {
                registry
                    .set_or_update_account_secondary_aggregation_key(
                        account_key,
                        secondary_aggregation_key.clone(),
                    )
                    .map_err(
                        ConfigExecutionError::RegistrySetOrUpdateSecondaryAggregationKeyError,
                    )?;
            }

            if let Some(projector_config) = config.projector_config {
                registry
                    .set_or_update_account_projector_config(account_key, projector_config)
                    .map_err(ConfigExecutionError::RegistrySetOrUpdateProjectorConfigError)?;
            }

            if let Some(flame_config) = &config.flame_config {
                registry
                    .set_or_update_account_flame_config(account_key, flame_config.clone())
                    .map_err(ConfigExecutionError::RegistrySetOrUpdateFlameConfigError)?;
            }
        }

        Ok(EntryFees::Config {
            base_fee,
            total_pre_subsidy: fees_pre_subsidy,
            subsidy_breakdown,
            secondary_aggregation_key_updated: config.secondary_aggregation_key.is_some(),
            projector_config_updated: config.projector_config.is_some(),
            flame_config_updated: config.flame_config.is_some(),
        })
    }

    async fn apply_subsidy_config(
        &self,
        account_key: [u8; 32],
        execution_timestamp: u64,
        fees_pre_subsidy: u64,
        latest_activity_timestamp: u64,
    ) -> Result<(u64, Option<ExemptionSubsidyBreakdown>), ConfigExecutionError> {
        let txfee_exemptions = {
            let _privileges_manager = self.privileges_manager.lock().await;
            _privileges_manager.get_account_txfee_exemptions(account_key)
        };

        let Some(mut exemptions) = txfee_exemptions else {
            return Ok((fees_pre_subsidy, None));
        };

        let bd = exemptions
            .apply_subsidy(
                execution_timestamp,
                latest_activity_timestamp,
                fees_pre_subsidy,
            )
            .ok_or(ConfigExecutionError::FailedToApplyFeesSubsidy)?;

        let fees_after_subsidy = bd.post_discount_leftover;

        {
            let mut _privileges_manager = self.privileges_manager.lock().await;
            match _privileges_manager
                .set_or_update_account_txfee_exemptions(account_key, exemptions)
            {
                Ok(()) => {}
                Err(PMUpdateAccountError::AccountIsNotPermanentlyRegistered(_)) => {}
            }
        }

        Ok((fees_after_subsidy, Some(bd)))
    }
}

async fn decrease_account_balance_with_coin_manager(
    coin_manager: &COIN_MANAGER,
    account_key: [u8; 32],
    amount: u64,
) -> Result<(), CMAccountBalanceDownError> {
    let mut _coin_manager = coin_manager.lock().await;
    _coin_manager.account_balance_down(account_key, amount)
}
