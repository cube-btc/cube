use crate::constructive::entity::account::root_account::unregistered_root_account::unregistered_root_account::UnregisteredRootAccount;
use crate::inscriptive::privileges_manager::bodies::account_body::account_body::PrivilegesManagerAccountBody;
use crate::inscriptive::privileges_manager::elements::account_hierarchy::account_hierarchy::AccountHierarchy;
use crate::inscriptive::privileges_manager::elements::exemption::exemption::Exemption;
use crate::inscriptive::privileges_manager::elements::exemption::periodic_resource::periodic_resource::PeriodicResource;
use crate::inscriptive::privileges_manager::elements::liveness_flag::liveness_flag::LivenessFlag;
use crate::inscriptive::privileges_manager::elements::timed_switch::timed_switch_bool::timed_switch_bool::TimedSwitchBool;
use crate::inscriptive::privileges_manager::privileges_manager::PRIVILEGES_MANAGER;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::flame_manager::flame_manager::FLAME_MANAGER;
use crate::constructive::entity::account::root_account::unregistered_root_account::ext::register_with_db::register_with_db_error::UnregisteredRootAccountRegisterWithDBError;
use crate::inscriptive::graveyard::graveyard::GRAVEYARD;

impl UnregisteredRootAccount {
    pub async fn register_with_db(
        &self,
        execution_timestamp: u64,
        registery: &REGISTERY,
        coin_manager: &COIN_MANAGER,
        flame_manager: &FLAME_MANAGER,
        privileges_manager: &PRIVILEGES_MANAGER,
        graveyard: &GRAVEYARD,
        initial_account_balance_in_satoshis: u64,
    ) -> Result<(), UnregisteredRootAccountRegisterWithDBError> {
        // 1 Check if the Account has been burried.
        {
            // 1.1 Lock the graveyard.
            let _graveyard = graveyard.lock().await;

            // 1.2 Check if the account has already been burried.
            if _graveyard.is_account_burried(self.account_key_to_be_registered) {
                return Err(UnregisteredRootAccountRegisterWithDBError::AccountHasBeenBurriedError);
            }
        }

        // 2 Register the account with the registery.
        {
            // 2.1 Lock the registery.
            let mut _registery = registery.lock().await;

            // 2.2 Register the account with the registery.
            _registery
                .register_account(
                    self.account_key_to_be_registered,
                    execution_timestamp,
                    Some(self.bls_key_to_be_configured),
                    None,
                    self.flame_config_to_be_configured.clone(),
                )
                .map_err(|e| {
                    UnregisteredRootAccountRegisterWithDBError::RegisteryRegisterAccountError(e)
                })?;
        }

        // 3 Register the account with the `CoinManager`.
        {
            // 3.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 3.2 Register the `RootAccount` with the `CoinManager`.
            _coin_manager
                .register_account(
                    self.account_key_to_be_registered,
                    initial_account_balance_in_satoshis,
                )
                .map_err(|e| {
                    UnregisteredRootAccountRegisterWithDBError::CoinManagerRegisterAccountError(e)
                })?;
        }

        // 4 Register the account with the `FlameManager`.
        {
            // 4.1 Lock the flame manager.
            let mut _flame_manager = flame_manager.lock().await;

            // 4.2 Register the `RootAccount` with the `FlameManager`.
            _flame_manager
                .register_account(self.account_key_to_be_registered)
                .map_err(|e| {
                    UnregisteredRootAccountRegisterWithDBError::FlameManagerRegisterAccountError(e)
                })?;
        }

        // 6 Register the account with the `PrivilegesManager`.
        {
            // 6.1 Construct privileges manager account body.
            let privileges_manager_account_body = PrivilegesManagerAccountBody::new(
                LivenessFlag::new_operational(),
                AccountHierarchy::new_pleb(),
                Exemption::new(PeriodicResource::new(100, 0, 0), 0, 0),
                TimedSwitchBool::new(true, None),
                TimedSwitchBool::new(true, None),
            );

            // 6.2 Register the account with the `PrivilegesManager`.
            let mut _privileges_manager = privileges_manager.lock().unwrap();
            _privileges_manager
                .register_account(
                    self.account_key_to_be_registered,
                    privileges_manager_account_body,
                )
                .map_err(
                    UnregisteredRootAccountRegisterWithDBError::PrivilegesManagerRegisterAccountError,
                )?;
        }

        // 7 Return the result.
        Ok(())
    }
}
