use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::entry::entry_fees::entry_fees::EntryFees;
use crate::constructive::entry::entry_kinds::deploy::deploy::Deploy;
use crate::executive::entry_executions::deploy_execution::error::deploy_execution_error::DeployExecutionError;
use crate::executive::exec_ctx::exec_ctx::ExecCtx;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::coin_manager::errors::balance_update_errors::CMAccountBalanceDownError;
use crate::inscriptive::privileges_manager::bodies::contract_body::contract_body::PrivilegesManagerContractBody;
use crate::inscriptive::privileges_manager::elements::exemption::exemption::Exemption;
use crate::inscriptive::privileges_manager::elements::exemption::exemption::ExemptionSubsidyBreakdown;
use crate::inscriptive::privileges_manager::elements::liveness_flag::liveness_flag::LivenessFlag;
use crate::inscriptive::privileges_manager::errors::update_error::PMUpdateAccountError;
use crate::executive::executable::compiler::compiler::ProgramCompiler;

impl ExecCtx {
    /// Executes a `Deploy` entry.
    pub async fn execute_deploy_internal(
        &mut self,
        deploy: &Deploy,
        execution_timestamp: u64,
    ) -> Result<EntryFees, DeployExecutionError> {
        let account_key = deploy.root_account.account_key();

        let params_holder = {
            let _params_manager = self._params_manager.lock().unwrap();
            _params_manager.get_params_holder()
        };
        let base_fee = params_holder.deploy_entry_base_fee;
        deploy
            .program
            .validate_methods()
            .map_err(DeployExecutionError::ProgramValidateMethodsError)?;
        let program_bytes_len = deploy
            .program
            .compile()
            .map_err(|_| DeployExecutionError::ProgramCompileError)?
            .len() as u64;
        let program_byte_fee = program_bytes_len
            .checked_mul(params_holder.deploy_entry_per_program_byte_fee)
            .ok_or(DeployExecutionError::DeployProgramByteFeeOverflow)?;
        let fees_pre_subsidy = base_fee
            .checked_add(program_byte_fee)
            .ok_or(DeployExecutionError::DeployTotalPreSubsidyOverflow)?;

        let latest_activity_timestamp = {
            let _registry = self.registry.lock().await;
            _registry
                .get_account_last_activity_timestamp(account_key)
                .unwrap_or(0)
        };

        let (fees_after_subsidy, subsidy_breakdown) = match &deploy.root_account {
            RootAccount::UnregisteredRootAccount(_) => {
                return Err(DeployExecutionError::UnexpectedUnregisteredRootAccountError);
            }
            RootAccount::RegisteredButUnconfiguredRootAccount(
                registered_but_unconfigured_root_account,
            ) => {
                if !registered_but_unconfigured_root_account.validate_bls_key() {
                    return Err(
                        DeployExecutionError::RegisteredButUnconfiguredRootAccountValidateBLSKeyError,
                    );
                }

                if !registered_but_unconfigured_root_account.verify_authorization_signature() {
                    return Err(
                        DeployExecutionError::RegisteredButUnconfiguredRootAccountInvalidAuthorizationSignatureError,
                    );
                }

                registered_but_unconfigured_root_account
                    .sync_with_registry(execution_timestamp, &self.registry)
                    .await
                    .map_err(
                        DeployExecutionError::RegisteredButUnconfiguredRootAccountSyncWithRegistryError,
                    )?;
                self.apply_subsidy_deploy(
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
                        DeployExecutionError::RegisteredAndConfiguredRootAccountSyncWithRegistryError,
                    )?;
                self.apply_subsidy_deploy(
                    account_key,
                    execution_timestamp,
                    fees_pre_subsidy,
                    latest_activity_timestamp,
                )
                .await?
            }
        };

        let total_debit = fees_after_subsidy
            .checked_add(deploy.initial_balance as u64)
            .ok_or(DeployExecutionError::DeployFeeDebitOverflow)?;
        
        decrease_account_balance_with_coin_manager(&self.coin_manager, account_key, total_debit)
            .await
            .map_err(DeployExecutionError::CoinManagerAccountBalanceDownError)?;

        let contract_id = deploy.program.contract_id();

        {
            let mut registry = self.registry.lock().await;
            registry
                .register_contract(contract_id, execution_timestamp, deploy.program.clone())
                .map_err(DeployExecutionError::RegistryRegisterContractError)?;
        }

        {
            let mut coin_manager = self.coin_manager.lock().await;
            coin_manager
                .register_contract(contract_id, deploy.initial_balance as u64)
                .map_err(DeployExecutionError::CoinManagerRegisterContractError)?;
        }

        {
            let mut state_manager = self.state_manager.lock().await;
            state_manager
                .register_contract(contract_id)
                .map_err(DeployExecutionError::StateManagerRegisterContractError)?;
        }

        {
            let contract_body = PrivilegesManagerContractBody::new(
                LivenessFlag::new_operational(),
                false,
                Exemption::new(None, None, None),
            );
            let mut privileges_manager = self.privileges_manager.lock().await;
            privileges_manager
                .register_contract(contract_id, contract_body)
                .map_err(DeployExecutionError::PrivilegesManagerRegisterContractError)?;
        }

        Ok(EntryFees::Deploy {
            base_fee,
            total_pre_subsidy: fees_pre_subsidy,
            subsidy_breakdown,
            initial_balance: deploy.initial_balance as u64,
            program_bytes_len,
            contract_id,
        })
    }

    async fn apply_subsidy_deploy(
        &self,
        account_key: [u8; 32],
        execution_timestamp: u64,
        fees_pre_subsidy: u64,
        latest_activity_timestamp: u64,
    ) -> Result<(u64, Option<ExemptionSubsidyBreakdown>), DeployExecutionError> {
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
            .ok_or(DeployExecutionError::FailedToApplyFeesSubsidy)?;

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
