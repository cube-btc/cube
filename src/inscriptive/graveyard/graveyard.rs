use crate::inscriptive::graveyard::delta::delta::GraveyardDelta;
use crate::inscriptive::graveyard::errors::apply_changes_error::GraveyardApplyChangesError;
use crate::inscriptive::graveyard::errors::bury_account_error::GraveyardBuryAccountError;
use crate::inscriptive::graveyard::errors::construction_error::GraveyardConstructionError;
use crate::inscriptive::graveyard::errors::redeem_account_coins_error::GraveyardRedeemAccountCoinsError;
use crate::operative::run_args::chain::Chain;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Account key.
type AccountKey = [u8; 32];

/// Redemption amount in satoshis.
type RedemptionAmountInSatoshis = u64;

/// Minimum redemption amount (dust).
pub const MIN_REDEMPTION_AMOUNT: u64 = 500;

/// Local storage manager for the storing destroyed accounts.
///
/// High Level Overview: The graveyard is used to store the accounts (plebs and residents) that have been destroyed.
/// Esentially, it is a list of accounts that have been destroyed and the amount of coins (BTC) they are owed for redemption.
/// The amount part is the variable part of the database which is updated to zero upon redemption.
/// Upon redemption, we reset the amount to zero, and still keep the records for historic-record-keeping so that they cannot be re-registered.
pub struct Graveyard {
    // In-memory buried accounts.
    in_memory_buried_accounts: HashMap<AccountKey, RedemptionAmountInSatoshis>,

    // On-disk db for storing the buried accounts.
    on_disk_buried_accounts: sled::Db,

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
        // 1 Open the graveyard db.
        let graveyard_db_path = format!("storage/{}/graveyard", chain.to_string());
        let graveyard_db =
            sled::open(graveyard_db_path).map_err(GraveyardConstructionError::DBOpenError)?;

        // 2 Initialize the in-memory buried accounts.
        let mut in_memory_buried_accounts =
            HashMap::<AccountKey, RedemptionAmountInSatoshis>::new();

        // 3 Iterate over all items in the graveyard db to collect the buried accounts.
        for lookup in graveyard_db.iter() {
            // 3.1 Get the key and value.
            if let Ok((key, val)) = lookup {
                // 3.1.1 Deserialize the account key.
                let account_key: [u8; 32] = key.as_ref().try_into().map_err(|_| {
                    GraveyardConstructionError::UnableToDeserializeAccountKeyBytesFromDBKey(
                        key.to_vec(),
                    )
                })?;

                // 3.1.2 Deserialize the coins redemption amount.
                let redemption_amount: u64 =
                    u64::from_le_bytes(val.as_ref().try_into().map_err(|_| {
                        GraveyardConstructionError::UnableToDeserializeRedemptionAmountBytesFromDBValue(
                            key.to_vec(),
                            val.to_vec(),
                        )
                    })?);

                // 3.1.3 Insert the buried account into the in-memory buried accounts.
                in_memory_buried_accounts.insert(account_key, redemption_amount);
            }
        }

        // 4 Construct the graveyard.
        let graveyard = Graveyard {
            in_memory_buried_accounts,
            on_disk_buried_accounts: graveyard_db,
            delta: GraveyardDelta::fresh_new(),
            backup_of_delta: GraveyardDelta::fresh_new(),
        };

        // 5 Guard the graveyard.
        let graveyard = Arc::new(Mutex::new(graveyard));

        // 6 Return the graveyard.
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
    ) -> Result<(), GraveyardBuryAccountError> {
        // 1 Check if the account has just been ephemerally buried.
        if self.delta.is_account_ephemerally_buried(account_key) {
            return Err(
                GraveyardBuryAccountError::AccountHasJustBeenEpheremallyBuried(account_key),
            );
        }

        // 2 Check if the account has already been permanently buried.
        if self.in_memory_buried_accounts.contains_key(&account_key) {
            return Err(
                GraveyardBuryAccountError::AccountIsAlreadyPermanentlyBuried(account_key),
            );
        }

        // 3 Epheremally bury the account in the delta.
        self.delta
            .ephemerally_bury_account(account_key, redemption_amount);

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

        // 3 Flush the delta.
        self.flush_deltas();

        // 4 Return success.
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

        // 3 Return the JSON object.
        Value::Object(obj)
    }
}

/// Erases the graveyard by db path.
pub fn erase_graveyard(chain: Chain) {
    // Graveyard db path.
    let graveyard_db_path = format!("storage/{}/graveyard", chain.to_string());

    // Erase the path.
    let _ = std::fs::remove_dir_all(graveyard_db_path);
}
