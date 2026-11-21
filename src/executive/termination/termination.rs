use crate::executive::exec_ctx::exec_ctx::ExecCtx;
use crate::executive::termination::error::termination_error::TerminationError;
use crate::inscriptive::privileges_manager::elements::account_hierarchy::account_hierarchy::AccountHierarchy;
use crate::inscriptive::privileges_manager::elements::liveness_flag::liveness_flag::LivenessFlag;

/// Parts-per-million denominator used for occupancy tax rate calculations.
const PPM_DENOMINATOR: u128 = 1_000_000;

impl ExecCtx {
    /// Executes a periodic termination.
    pub(crate) async fn periodic_termination(
        &mut self,
        current_execution_batch_height: u64,
        current_execution_timestamp: u64,
    ) -> Result<(), TerminationError> {
        // 1 Get params from the params manager.
        let params_holder = {
            let _params_manager = self._params_manager.lock().await;
            _params_manager.get_params_holder()
        };

        // 2 Get certain related parameters from the params holder.
        let (
            account_termination_check_period,
            contract_termination_check_period,
            configured_pleb_inactivity_termination_period_seconds,
            unconfigured_pleb_inactivity_termination_period_seconds,
            configured_pleb_to_be_destroyed_recovery_gap_seconds,
            unconfigured_pleb_to_be_destroyed_recovery_gap_seconds,
            mutable_contract_vitality_sucked_termination_value,
            mutable_contract_to_be_destroyed_gap_seconds,
            contract_binsize_ppm_tax_rate,
            contract_statesize_ppm_tax_rate,
            contract_shadowed_liquidity_ppm_tax_rate,
            contract_non_shadowed_liquidity_ppm_tax_rate,
        ) = (
            params_holder.account_termination_check_period,
            params_holder.contract_termination_check_period,
            params_holder.configured_pleb_inactivity_termination_period_seconds,
            params_holder.unconfigured_pleb_inactivity_termination_period_seconds,
            params_holder.configured_pleb_to_be_destroyed_recovery_gap_seconds,
            params_holder.unconfigured_pleb_to_be_destroyed_recovery_gap_seconds,
            params_holder.mutable_contract_vitality_sucked_termination_value,
            params_holder.mutable_contract_to_be_destroyed_gap_seconds,
            params_holder.contract_binsize_ppm_tax_rate,
            params_holder.contract_statesize_ppm_tax_rate,
            params_holder.contract_shadowed_liquidity_ppm_tax_rate,
            params_holder.contract_non_shadowed_liquidity_ppm_tax_rate,
        );

        // 3 Get the latest cube batch height periodic check tips from the sync manager.
        let (
            latest_account_period_check_cube_batch_height_tip,
            latest_contract_period_check_cube_batch_height_tip,
        ) = {
            let sync_manager = self.sync_manager.lock().await;
            (
                sync_manager.latest_account_period_check_cube_batch_height_tip(),
                sync_manager.latest_contract_period_check_cube_batch_height_tip(),
            )
        };

        // 4 Accounts Phase:
        // Check for plebs that have not been active for some period. Then set their 'LivenessFlag' to 'ToBeDestroyed'.
        // Then destroy the much-earlier-set ToBeDestroyed Plebs in another iteration.
        {
            // 4.1 Check if this is an appropriate period for an account period check.
            if (latest_account_period_check_cube_batch_height_tip
                + account_termination_check_period)
                <= current_execution_batch_height
            {
                // 4.1.1 Destroy earlier-marked ToBeDestroyed plebs/residents
                // whose TakesEffectAtTimestamp has elapsed.
                {
                    let mut plebs_and_residents_to_destroy = Vec::<[u8; 32]>::new();
                    {
                        let privileges_manager = self.privileges_manager.lock().await;

                        for account_key in privileges_manager.account_keys() {
                            let Some(hierarchy) =
                                privileges_manager.get_account_hierarchy(*account_key)
                            else {
                                continue;
                            };
                            if !matches!(
                                hierarchy,
                                AccountHierarchy::Pleb | AccountHierarchy::Resident
                            ) {
                                continue;
                            }

                            let Some(liveness_flag) =
                                privileges_manager.get_account_liveness_flag(*account_key)
                            else {
                                continue;
                            };
                            if let LivenessFlag::ToBeDestroyed(takes_effect_at_timestamp) =
                                liveness_flag
                            {
                                if takes_effect_at_timestamp < current_execution_timestamp {
                                    plebs_and_residents_to_destroy.push(*account_key);
                                }
                            }
                        }
                    }

                    for account_key in plebs_and_residents_to_destroy {
                        self.destroy_account(account_key).await?;
                    }
                }

                // 4.1.2 Mark inactive operational plebs as ToBeDestroyed.
                // Newly marked accounts are destroyed in a later period-check iteration
                // once current_execution_timestamp + ToBeDestroyed recovery gap has elapsed.
                {
                    // (account_key, takes_effect_at_timestamp)
                    let mut plebs_to_mark = Vec::<([u8; 32], u64)>::new();
                    {
                        let privileges_manager = self.privileges_manager.lock().await;
                        let registry = self.registry.lock().await;

                        for account_key in privileges_manager.account_keys() {
                            let Some(hierarchy) =
                                privileges_manager.get_account_hierarchy(*account_key)
                            else {
                                continue;
                            };
                            if !matches!(hierarchy, AccountHierarchy::Pleb) {
                                continue;
                            }

                            let Some(liveness_flag) =
                                privileges_manager.get_account_liveness_flag(*account_key)
                            else {
                                continue;
                            };
                            if !matches!(liveness_flag, LivenessFlag::Operational) {
                                continue;
                            }

                            let Some(last_activity_timestamp) =
                                registry.get_account_last_activity_timestamp(*account_key)
                            else {
                                continue;
                            };
                            let is_configured = registry
                                .get_account_body_by_account_key(*account_key)
                                .map(|body| body.primary_bls_key.is_some())
                                .unwrap_or(false);

                            let inactivity_period = if is_configured {
                                configured_pleb_inactivity_termination_period_seconds
                            } else {
                                unconfigured_pleb_inactivity_termination_period_seconds
                            };

                            if last_activity_timestamp.saturating_add(inactivity_period)
                                < current_execution_timestamp
                            {
                                let to_be_destroyed_recovery_gap = if is_configured {
                                    configured_pleb_to_be_destroyed_recovery_gap_seconds
                                } else {
                                    unconfigured_pleb_to_be_destroyed_recovery_gap_seconds
                                };
                                let takes_effect_at_timestamp = current_execution_timestamp
                                    .saturating_add(to_be_destroyed_recovery_gap);
                                plebs_to_mark.push((*account_key, takes_effect_at_timestamp));
                            }
                        }
                    }

                    if !plebs_to_mark.is_empty() {
                        let mut privileges_manager = self.privileges_manager.lock().await;
                        for (account_key, takes_effect_at_timestamp) in plebs_to_mark {
                            privileges_manager
                                .set_or_update_account_liveness_flag(
                                    account_key,
                                    LivenessFlag::new_to_be_destroyed(takes_effect_at_timestamp),
                                )
                                .map_err(|error| {
                                    TerminationError::UnableToUpdateAccountLivenessFlag(
                                        account_key,
                                        error,
                                    )
                                })?;
                        }
                    }
                }

                // 4.1.3 Update the latest account period check tip.
                {
                    let mut sync_manager = self.sync_manager.lock().await;
                    sync_manager.set_latest_account_period_check_cube_batch_height_tip(
                        current_execution_batch_height,
                    );
                }
            }
        }

        // 5 Contracts Phase:
        // Check mutable contracts whose vitality is sucked below the termination threshold,
        // mark them ToBeDestroyed, then destroy earlier-marked contracts whose gap has elapsed.
        {
            // 5.1 Check if this is an appropriate period for a contract period check.
            if (latest_contract_period_check_cube_batch_height_tip
                + contract_termination_check_period)
                <= current_execution_batch_height
            {
                // 5.1.1 Destroy earlier-marked ToBeDestroyed mutable contracts
                // whose TakesEffectAtTimestamp has elapsed.
                {
                    let mut mutable_contracts_to_destroy = Vec::<[u8; 32]>::new();
                    {
                        let privileges_manager = self.privileges_manager.lock().await;

                        for contract_id in privileges_manager.contract_ids() {
                            let Some(immutability) =
                                privileges_manager.get_contract_immutability(*contract_id)
                            else {
                                continue;
                            };
                            // Only mutable contracts.
                            if immutability {
                                continue;
                            }

                            let Some(liveness_flag) =
                                privileges_manager.get_contract_liveness_flag(*contract_id)
                            else {
                                continue;
                            };
                            if let LivenessFlag::ToBeDestroyed(takes_effect_at_timestamp) =
                                liveness_flag
                            {
                                if takes_effect_at_timestamp < current_execution_timestamp {
                                    mutable_contracts_to_destroy.push(*contract_id);
                                }
                            }
                        }
                    }

                    for contract_id in mutable_contracts_to_destroy {
                        self.destroy_contract(contract_id).await?;
                    }
                }

                // 5.1.2 Apply occupancy tax to mutable contracts by sucking vitality.
                // Tax is derived from binsize, statesize, shadowed and non-shadowed liquidity PPM rates.
                {
                    // (contract_id, tax_in_satoshis)
                    let mut mutable_contracts_to_tax = Vec::<([u8; 32], u64)>::new();
                    {
                        let privileges_manager = self.privileges_manager.lock().await;
                        let coin_manager = self.coin_manager.lock().await;
                        let registry = self.registry.lock().await;
                        let state_manager = self.state_manager.lock().await;

                        for contract_id in privileges_manager.contract_ids() {
                            let Some(immutability) =
                                privileges_manager.get_contract_immutability(*contract_id)
                            else {
                                continue;
                            };
                            // Only mutable contracts pay vitality occupancy tax.
                            if immutability {
                                continue;
                            }

                            let binsize_bytes = registry
                                .get_contract_binsize_in_bytes(*contract_id)
                                .unwrap_or(0);
                            let statesize_bytes =
                                state_manager.get_contract_state_size_in_bytes(*contract_id);
                            let shadowed_liquidity_sats = coin_manager
                                .get_contract_shadow_allocs_sum_in_satoshis(*contract_id)
                                .unwrap_or(0);
                            let contract_balance_sats =
                                coin_manager.get_contract_balance(*contract_id).unwrap_or(0);
                            let non_shadowed_liquidity_sats =
                                contract_balance_sats.saturating_sub(shadowed_liquidity_sats);

                            let binsize_tax = (binsize_bytes as u128)
                                .saturating_mul(contract_binsize_ppm_tax_rate as u128)
                                / PPM_DENOMINATOR;
                            let statesize_tax = (statesize_bytes as u128)
                                .saturating_mul(contract_statesize_ppm_tax_rate as u128)
                                / PPM_DENOMINATOR;
                            let shadowed_liquidity_tax = (shadowed_liquidity_sats as u128)
                                .saturating_mul(contract_shadowed_liquidity_ppm_tax_rate as u128)
                                / PPM_DENOMINATOR;
                            let non_shadowed_liquidity_tax =
                                (non_shadowed_liquidity_sats as u128).saturating_mul(
                                    contract_non_shadowed_liquidity_ppm_tax_rate as u128,
                                ) / PPM_DENOMINATOR;

                            let total_tax = binsize_tax
                                .saturating_add(statesize_tax)
                                .saturating_add(shadowed_liquidity_tax)
                                .saturating_add(non_shadowed_liquidity_tax)
                                as u64;

                            if total_tax > 0 {
                                mutable_contracts_to_tax.push((*contract_id, total_tax));
                            }
                        }
                    }

                    if !mutable_contracts_to_tax.is_empty() {
                        let mut coin_manager = self.coin_manager.lock().await;
                        for (contract_id, tax_in_satoshis) in mutable_contracts_to_tax {
                            coin_manager
                                .contract_vitality_balance_down(contract_id, tax_in_satoshis)
                                .map_err(|error| {
                                    TerminationError::UnableToApplyContractVitalityTax(
                                        contract_id,
                                        error,
                                    )
                                })?;
                        }
                    }
                }

                // 5.1.3 Mark vitality-sucked operational mutable contracts as ToBeDestroyed.
                // Newly marked contracts are destroyed in a later period-check iteration
                // once current_execution_timestamp + ToBeDestroyed gap has elapsed.
                {
                    // (contract_id, takes_effect_at_timestamp)
                    let mut mutable_contracts_to_mark = Vec::<([u8; 32], u64)>::new();
                    {
                        let privileges_manager = self.privileges_manager.lock().await;
                        let coin_manager = self.coin_manager.lock().await;

                        for contract_id in privileges_manager.contract_ids() {
                            let Some(immutability) =
                                privileges_manager.get_contract_immutability(*contract_id)
                            else {
                                continue;
                            };
                            // Only mutable contracts.
                            if immutability {
                                continue;
                            }

                            let Some(liveness_flag) =
                                privileges_manager.get_contract_liveness_flag(*contract_id)
                            else {
                                continue;
                            };
                            if !matches!(liveness_flag, LivenessFlag::Operational) {
                                continue;
                            }

                            let Some(vitality_balance) =
                                coin_manager.get_contract_vitality_balance(*contract_id)
                            else {
                                continue;
                            };

                            // Vitality has been sucked at/below the termination threshold.
                            if vitality_balance
                                <= mutable_contract_vitality_sucked_termination_value
                            {
                                let takes_effect_at_timestamp = current_execution_timestamp
                                    .saturating_add(mutable_contract_to_be_destroyed_gap_seconds);
                                mutable_contracts_to_mark
                                    .push((*contract_id, takes_effect_at_timestamp));
                            }
                        }
                    }

                    if !mutable_contracts_to_mark.is_empty() {
                        let mut privileges_manager = self.privileges_manager.lock().await;
                        for (contract_id, takes_effect_at_timestamp) in mutable_contracts_to_mark {
                            privileges_manager
                                .set_or_update_contract_liveness_flag(
                                    contract_id,
                                    LivenessFlag::new_to_be_destroyed(takes_effect_at_timestamp),
                                )
                                .map_err(|error| {
                                    TerminationError::UnableToUpdateContractLivenessFlag(
                                        contract_id,
                                        error,
                                    )
                                })?;
                        }
                    }
                }

                // 5.1.4 Update the latest contract period check tip.
                {
                    let mut sync_manager = self.sync_manager.lock().await;
                    sync_manager.set_latest_contract_period_check_cube_batch_height_tip(
                        current_execution_batch_height,
                    );
                }
            }
        }

        Ok(())
    }

    /// Destroys an account across all managers that hold account state.
    ///
    /// Ephemeral erasures are persisted later via `ExecCtx::apply_changes`.
    async fn destroy_account(&mut self, account_key: [u8; 32]) -> Result<(), TerminationError> {
        // 1 Resolve redemption amount (owned balance + owed shadow value).
        let redemption_amount = {
            let coin_manager = self.coin_manager.lock().await;
            coin_manager
                .get_account_target_flame_value_in_satoshis(account_key)
                .or_else(|| coin_manager.get_account_balance(account_key))
                .unwrap_or(0)
        };

        // 2 Bury the account in the graveyard for coin redemption.
        {
            let mut graveyard = self.graveyard.lock().await;
            graveyard
                .bury_account(account_key, redemption_amount, true)
                .map_err(|error| TerminationError::UnableToBuryAccount(account_key, error))?;
        }

        // 3 Erase from the coin manager.
        {
            let mut coin_manager = self.coin_manager.lock().await;
            if coin_manager.is_account_permanently_registered(account_key) {
                coin_manager
                    .erase_account(account_key, true)
                    .map_err(|error| {
                        TerminationError::UnableToEraseAccountFromCoinManager(account_key, error)
                    })?;
            }
        }

        // 4 Erase from the flame manager if present.
        {
            let mut flame_manager = self.flame_manager.lock().await;
            if flame_manager.is_account_permanently_registered(account_key) {
                flame_manager
                    .erase_account(account_key, true)
                    .map_err(|error| {
                        TerminationError::UnableToEraseAccountFromFlameManager(account_key, error)
                    })?;
            }
        }

        // 5 Erase from the privileges manager.
        {
            let mut privileges_manager = self.privileges_manager.lock().await;
            privileges_manager
                .erase_account(account_key, true)
                .map_err(|error| {
                    TerminationError::UnableToEraseAccountFromPrivilegesManager(account_key, error)
                })?;
        }

        // 6 Erase from the registry.
        {
            let mut registry = self.registry.lock().await;
            registry.erase_account(account_key).map_err(|error| {
                TerminationError::UnableToEraseAccountFromRegistry(account_key, error)
            })?;
        }

        Ok(())
    }

    /// Destroys a contract across all managers that hold contract state.
    ///
    /// Ephemeral erasures are persisted later via `ExecCtx::apply_changes`.
    async fn destroy_contract(&mut self, contract_id: [u8; 32]) -> Result<(), TerminationError> {
        // 1 Bury the contract in the graveyard (historic record; no coin redemption).
        {
            let mut graveyard = self.graveyard.lock().await;
            graveyard
                .bury_contract(contract_id)
                .map_err(|error| TerminationError::UnableToBuryContract(contract_id, error))?;
        }

        // 2 Erase from the coin manager.
        {
            let mut coin_manager = self.coin_manager.lock().await;
            if coin_manager.is_contract_permanently_registered(contract_id) {
                coin_manager
                    .erase_contract(contract_id, true)
                    .map_err(|error| {
                        TerminationError::UnableToEraseContractFromCoinManager(contract_id, error)
                    })?;
            }
        }

        // 3 Erase from the state manager.
        {
            let mut state_manager = self.state_manager.lock().await;
            if state_manager.is_contract_permanently_registered(contract_id) {
                state_manager
                    .erase_contract(contract_id, true)
                    .map_err(|error| {
                        TerminationError::UnableToEraseContractFromStateManager(contract_id, error)
                    })?;
            }
        }

        // 4 Erase from the privileges manager.
        {
            let mut privileges_manager = self.privileges_manager.lock().await;
            privileges_manager
                .erase_contract(contract_id, true)
                .map_err(|error| {
                    TerminationError::UnableToEraseContractFromPrivilegesManager(contract_id, error)
                })?;
        }

        // 5 Erase from the registry.
        {
            let mut registry = self.registry.lock().await;
            registry.erase_contract(contract_id).map_err(|error| {
                TerminationError::UnableToEraseContractFromRegistry(contract_id, error)
            })?;
        }

        Ok(())
    }
}
