use crate::inscriptive::graveyard::delta::delta::GraveyardDelta;
use crate::inscriptive::graveyard::errors::apply_changes_error::GraveyardApplyChangesError;
use crate::inscriptive::graveyard::errors::bury_account_error::GraveyardBuryAccountError;
use crate::inscriptive::graveyard::errors::bury_contract_error::GraveyardBuryContractError;
use crate::inscriptive::graveyard::errors::construction_error::GraveyardConstructionError;
use crate::inscriptive::graveyard::errors::redeem_account_coins_error::GraveyardRedeemAccountCoinsError;
use crate::operative::run_args::chain::Chain;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Account key.
type AccountKey = [u8; 32];

/// Contract id.
type ContractId = [u8; 32];

/// Redemption amount in satoshis.
type RedemptionAmountInSatoshis = u64;

/// Minimum redemption amount (dust).
pub const MIN_REDEMPTION_AMOUNT: u64 = 500;

/// Local storage manager for the storing destroyed accounts and contracts.
///
/// High Level Overview: The graveyard is used to store the accounts (plebs and residents) and contracts
/// that have been destroyed.
/// Essentially, it is a list of accounts that have been destroyed and the amount of coins (BTC) they are owed for redemption,
/// plus a list of destroyed contracts kept for historic-record-keeping so that they cannot be re-registered.
/// The amount part is the variable part of the database which is updated to zero upon redemption.
/// Upon redemption, we reset the amount to zero, and still keep the records for historic-record-keeping so that they cannot be re-registered.
pub struct Graveyard {
    // In-memory buried accounts.
    in_memory_buried_accounts: HashMap<AccountKey, RedemptionAmountInSatoshis>,

    // In-memory buried contracts.
    in_memory_buried_contracts: Vec<ContractId>,

    // On-disk db for storing the buried accounts.
    on_disk_buried_accounts: sled::Db,

    // On-disk db for storing the buried contracts.
    on_disk_buried_contracts: sled::Db,

    // State differences to be applied.
    delta: GraveyardDelta,

    // Backup of state differences in case of rollback.
    backup_of_delta: GraveyardDelta,
}

/// Guarded 'Graveyard'.
#[allow(non_camel_case_types)]
pub type GRAVEYARD = Arc<Mutex<Graveyard>>;

impl Graveyard {
    pub fn new(chain: Chain) -> Result<GRAVEYARD, GraveyardConstructionError> {
        // 1 Open the buried accounts db.
        let accounts_db_path = format!("storage/{}/graveyard/accounts", chain);
        let on_disk_buried_accounts = sled::open(accounts_db_path)
            .map_err(GraveyardConstructionError::AccountsDBOpenError)?;

        // 2 Open the buried contracts db.
        let contracts_db_path = format!("storage/{}/graveyard/contracts", chain);
        let on_disk_buried_contracts = sled::open(contracts_db_path)
            .map_err(GraveyardConstructionError::ContractsDBOpenError)?;

        // 3 Initialize the in-memory buried accounts.
        let mut in_memory_buried_accounts =
            HashMap::<AccountKey, RedemptionAmountInSatoshis>::new();

        // 4 Iterate over all items in the buried accounts db.
        for (key, val) in on_disk_buried_accounts.iter().flatten() {
            // 4.1.1 Deserialize the account key.
            let account_key: [u8; 32] = key.as_ref().try_into().map_err(|_| {
                GraveyardConstructionError::UnableToDeserializeAccountKeyBytesFromDBKey(
                    key.to_vec(),
                )
            })?;

            // 4.1.2 Deserialize the coins redemption amount.
            let redemption_amount: u64 =
                u64::from_le_bytes(val.as_ref().try_into().map_err(|_| {
                    GraveyardConstructionError::UnableToDeserializeRedemptionAmountBytesFromDBValue(
                        key.to_vec(),
                        val.to_vec(),
                    )
                })?);

            // 4.1.3 Insert the buried account into the in-memory buried accounts.
            in_memory_buried_accounts.insert(account_key, redemption_amount);
        }

        // 5 Initialize the in-memory buried contracts.
        let mut in_memory_buried_contracts = Vec::<ContractId>::new();

        // 6 Iterate over all items in the buried contracts db.
        for (key, _) in on_disk_buried_contracts.iter().flatten() {
            // 6.1 Deserialize the contract id.
            let contract_id: [u8; 32] = key.as_ref().try_into().map_err(|_| {
                GraveyardConstructionError::UnableToDeserializeContractIdBytesFromDBKey(
                    key.to_vec(),
                )
            })?;

            // 6.2 Insert the buried contract into the in-memory buried contracts.
            in_memory_buried_contracts.push(contract_id);
        }

        // 7 Construct the graveyard.
        let graveyard = Graveyard {
            in_memory_buried_accounts,
            in_memory_buried_contracts,
            on_disk_buried_accounts,
            on_disk_buried_contracts,
            delta: GraveyardDelta::fresh_new(),
            backup_of_delta: GraveyardDelta::fresh_new(),
        };

        // 8 Guard the graveyard.
        let graveyard = Arc::new(Mutex::new(graveyard));

        // 9 Return the graveyard.
        Ok(graveyard)
    }

    /// Clones the delta into the backup.
    fn backup_delta(&mut self) {
        self.backup_of_delta = self.delta.clone();
    }

    /// Restores the delta from the backup.
    fn restore_delta(&mut self) {
        self.delta = self.backup_of_delta.clone();
    }

    /// Prepares the graveyard prior to each execution.
    ///
    /// NOTE: Used by the Engine.
    pub fn pre_execution(&mut self) {
        self.backup_delta();
    }

    /// Checks if an account is buried.
    pub fn is_account_buried(&self, account_key: [u8; 32]) -> bool {
        // 1 Check in the delta first.
        if self.delta.is_account_ephemerally_buried(account_key) {
            return true;
        }

        // 2 Check in the in-memory buried accounts.
        if self.in_memory_buried_accounts.contains_key(&account_key) {
            return true;
        }

        // 3 Return false.
        false
    }

    /// Checks if a contract is buried.
    pub fn is_contract_buried(&self, contract_id: [u8; 32]) -> bool {
        // 1 Check in the delta first.
        if self.delta.is_contract_ephemerally_buried(contract_id) {
            return true;
        }

        // 2 Check in the in-memory buried contracts.
        if self.in_memory_buried_contracts.contains(&contract_id) {
            return true;
        }

        // 3 Return false.
        false
    }

    /// Returns the redemption amount for an account.
    pub fn get_redemption_amount(&self, account_key: [u8; 32]) -> Option<u64> {
        // 1 If the account has just been ephemerally redeemed, redemption amount is now zero.
        if self.delta.is_account_ephemerally_redeemed(account_key) {
            return Some(0);
        }

        // 2 If the account has just been ephemerally buried:
        if self.delta.is_account_ephemerally_buried(account_key) {
            return self.delta.accounts_to_bury.get(&account_key).cloned();
        }

        // 2 Check in the in-memory buried accounts.
        self.in_memory_buried_accounts.get(&account_key).cloned()
    }

    /// Burries an account to the graveyard with their corresponding owed redemption amount in satoshis.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn bury_account(
        &mut self,
        account_key: [u8; 32],
        redemption_amount: u64,
        optimized: bool,
    ) -> Result<(), GraveyardBuryAccountError> {
        // 1 Check if the account has just been ephemerally buried.
        if self.delta.is_account_ephemerally_buried(account_key) {
            return Err(
                GraveyardBuryAccountError::AccountHasJustBeenEpheremallyBuried(account_key),
            );
        }

        // 2 Check if the account has already been permanently buried.
        if !optimized {
            if self.in_memory_buried_accounts.contains_key(&account_key) {
                return Err(
                    GraveyardBuryAccountError::AccountIsAlreadyPermanentlyBuried(account_key),
                );
            }
        }

        // 3 Epheremally bury the account in the delta.
        self.delta
            .ephemerally_bury_account(account_key, redemption_amount);

        // 4 Return the result.
        Ok(())
    }

    /// Buries a contract to the graveyard.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    /// NOTE: Contracts have no coin redemption.
    pub fn bury_contract(
        &mut self,
        contract_id: [u8; 32],
    ) -> Result<(), GraveyardBuryContractError> {
        // 1 Check if the contract has just been ephemerally buried.
        if self.delta.is_contract_ephemerally_buried(contract_id) {
            return Err(
                GraveyardBuryContractError::ContractHasJustBeenEphemerallyBuried(contract_id),
            );
        }

        // 2 Check if the contract has already been permanently buried.
        if self.in_memory_buried_contracts.contains(&contract_id) {
            return Err(
                GraveyardBuryContractError::ContractIsAlreadyPermanentlyBuried(contract_id),
            );
        }

        // 3 Ephemerally bury the contract in the delta.
        self.delta.ephemerally_bury_contract(contract_id);

        // 4 Return the result.
        Ok(())
    }

    /// Redeems the account coins and returns the redemption amount.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn redeem_account_coins(
        &mut self,
        account_key: [u8; 32],
    ) -> Result<u64, GraveyardRedeemAccountCoinsError> {
        // 1 Check if the account has just been ephemerally redeemed.
        if self.delta.is_account_ephemerally_redeemed(account_key) {
            return Err(
                GraveyardRedeemAccountCoinsError::AccountCoinsHasJustBeenEphemerallyRedeemed(
                    account_key,
                ),
            );
        }

        // 2 Check if the account has just been ephemerally buried.
        // We do not allow coin redemption upon burying the account in the same execution. Must wait for the next state transition.
        if self.delta.is_account_ephemerally_buried(account_key) {
            return Err(
                GraveyardRedeemAccountCoinsError::ThisAccountHasJustBeenEphemerallyBuried(
                    account_key,
                ),
            );
        }

        // 3 Get the redemption amount of the account.
        let redemption_amount = self
            .in_memory_buried_accounts
            .get(&account_key)
            .cloned()
            .ok_or(
            GraveyardRedeemAccountCoinsError::CouldNotRetrieveRedemptionAmountBecauseTheAccountIsNotBuried(
                account_key,
            ),
        )?;

        // 4 Check if the redemption amount is less than the minimum redemption amount.
        if redemption_amount < MIN_REDEMPTION_AMOUNT {
            return Err(
                GraveyardRedeemAccountCoinsError::RedemptionAmountIsLessThanTheMinimumLimit(
                    account_key,
                    redemption_amount,
                    MIN_REDEMPTION_AMOUNT,
                ),
            );
        }

        // 5 Epheremally redeem the account coins in the delta.
        self.delta
            .ephemerally_redeem_account_coins(account_key, redemption_amount);

        // 6 Return the result.
        Ok(redemption_amount)
    }

    /// Reverts the ephemeral changes associated with the last execution.
    pub fn rollback_last(&mut self) {
        self.restore_delta();
    }

    /// Applies the changes to the graveyard.
    ///
    /// This persists the delta to both in-memory and on-disk storage:
    /// - Redemptions: Resets SatoshiRedemptionAmount to zero for redeemed accounts.
    /// - New accounts: Adds accounts from `accounts_to_bury` to the buried accounts.
    /// - New contracts: Adds contracts from `contracts_to_bury` to the buried contracts.
    pub fn apply_changes(&mut self) -> Result<(), GraveyardApplyChangesError> {
        // 1 Apply redemptions: Reset redemption amounts to zero in-memory and on-disk.
        for (account_key, _) in self.delta.redemptions.iter() {
            // 1.1 Construct the zero redemption amount.
            let zero_redemption_amount: [u8; 8] = 0u64.to_le_bytes();

            // 1.2 On-disk: Reset the redemption amount to zero.
            self.on_disk_buried_accounts
                .insert(account_key, zero_redemption_amount.to_vec())
                .map_err(|e| {
                    GraveyardApplyChangesError::RedemptionAmountResetDBError(*account_key, e)
                })?;

            // 1.3 In-memory: Reset the redemption amount to zero.
            self.in_memory_buried_accounts.insert(*account_key, 0);
        }

        // 2 Insert new accounts to bury into in-memory and on-disk.
        for (account_key, redemption_amount) in self.delta.accounts_to_bury.iter() {
            // 2.1 On-disk: Insert the buried account.
            self.on_disk_buried_accounts
                .insert(account_key, redemption_amount.to_le_bytes().to_vec())
                .map_err(|e| {
                    GraveyardApplyChangesError::BuriedAccountInsertDBError(*account_key, e)
                })?;

            // 2.2 In-memory: Insert the buried account.
            self.in_memory_buried_accounts
                .insert(*account_key, *redemption_amount);
        }

        // 3 Insert new contracts to bury into in-memory and on-disk.
        for contract_id in self.delta.contracts_to_bury.iter() {
            // 3.1 On-disk: Insert the buried contract (no redemption amount).
            self.on_disk_buried_contracts
                .insert(contract_id, vec![])
                .map_err(|e| {
                    GraveyardApplyChangesError::BuriedContractInsertDBError(*contract_id, e)
                })?;

            // 3.2 In-memory: Insert the buried contract.
            self.in_memory_buried_contracts.push(*contract_id);
        }

        // 4 Flush the delta.
        self.flush_deltas();

        // 5 Return success.
        Ok(())
    }

    /// Clears all ephemeral changes from the delta.
    pub fn flush_deltas(&mut self) {
        self.delta.flush();
        self.backup_of_delta.flush();
    }

    /// Returns the graveyard as a JSON object.
    pub fn json(&self) -> Value {
        // 1 Construct the graveyard JSON object.
        let mut obj = Map::new();

        // 2 Insert the in-memory buried accounts.
        obj.insert(
            "buried_accounts".to_string(),
            Value::Object(
                self.in_memory_buried_accounts
                    .iter()
                    .map(|(account_key, redemption_amount)| {
                        (
                            hex::encode(account_key),
                            Value::Number(serde_json::Number::from(*redemption_amount)),
                        )
                    })
                    .collect(),
            ),
        );

        // 3 Insert the in-memory buried contracts.
        obj.insert(
            "buried_contracts".to_string(),
            Value::Array(
                self.in_memory_buried_contracts
                    .iter()
                    .map(|contract_id| Value::String(hex::encode(contract_id)))
                    .collect(),
            ),
        );

        // 4 Return the JSON object.
        Value::Object(obj)
    }
}

/// Erases the graveyard by db path.
pub fn erase_graveyard(chain: Chain) {
    // Graveyard db path (accounts + contracts).
    let graveyard_db_path = format!("storage/{}/graveyard", chain);

    // Erase the path.
    let _ = std::fs::remove_dir_all(graveyard_db_path);
}
