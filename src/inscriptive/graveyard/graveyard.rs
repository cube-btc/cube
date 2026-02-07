use crate::inscriptive::graveyard::delta::delta::GraveyardDelta;
use crate::inscriptive::graveyard::errors::apply_changes_error::GraveyardApplyChangesError;
use crate::inscriptive::graveyard::errors::burry_account_error::GraveyardBurryAccountError;
use crate::inscriptive::graveyard::errors::construction_error::GraveyardConstructionError;
use crate::inscriptive::graveyard::errors::redeem_account_coins_error::GraveyardRedeemAccountCoinsError;
use crate::operative::Chain;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Account key.
type AccountKey = [u8; 32];

/// Satoshi redemption amount.
type SatoshiRedemptionAmount = u64;

/// Minimum redemption amount (dust).
pub const MIN_REDEMPTION_AMOUNT: u64 = 500;

/// Local storage manager for the storing destroyed accounts.
pub struct Graveyard {
    // In-memory burried accounts.
    in_memory_burried_accounts: HashMap<AccountKey, SatoshiRedemptionAmount>,

    // On-disk db for storing the burried accounts.
    on_disk_burried_accounts: sled::Db,

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

        // 2 Initialize the in-memory burried accounts.
        let mut in_memory_burried_accounts = HashMap::<AccountKey, SatoshiRedemptionAmount>::new();

        // 3 Iterate over all items in the graveyard db to collect the burried accounts.
        for lookup in graveyard_db.iter() {
            // 3.1 Get the key and value.
            if let Ok((key, val)) = lookup {
                // 3.1.1 Deserialize the account key.
                let account_key: [u8; 32] = key.as_ref().try_into().map_err(|_| {
                    GraveyardConstructionError::UnableToDeserializeAccountKeyBytesFromTreeName(
                        key.to_vec(),
                    )
                })?;

                // 3.1.2 Deserialize the satoshi redemption amount.
                let satoshi_redemption_amount: u64 =
                    u64::from_le_bytes(val.as_ref().try_into().map_err(|_| {
                        GraveyardConstructionError::UnableToDeserializeSatoshiRedemptionAmountBytesFromTreeValue(
                            key.to_vec(),
                            val.to_vec(),
                        )
                    })?);

                // 3.1.3 Insert the burried account into the in-memory burried accounts.
                in_memory_burried_accounts.insert(account_key, satoshi_redemption_amount);
            }
        }

        // 4 Construct the graveyard.
        let graveyard = Graveyard {
            in_memory_burried_accounts,
            on_disk_burried_accounts: graveyard_db,
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

    /// Checks if an account is burried.
    pub fn is_account_burried(&self, account_key: [u8; 32]) -> bool {
        // 1 Check in the delta first.
        if self.delta.is_account_epheremally_burried(account_key) {
            return true;
        }

        // 2 Check in the in-memory burried accounts.
        if self.in_memory_burried_accounts.contains_key(&account_key) {
            return true;
        }

        // 3 Return false.
        false
    }

    /// Returns the redemption amount for an account.
    pub fn get_redemption_amount(&self, account_key: [u8; 32]) -> Option<u64> {
        // 1 If the account has just been epheremally redeemed, redemption amount is now zero.
        if self.delta.is_account_epheremally_redeemed(account_key) {
            return Some(0);
        }

        // 2 If the account has just been epheremally burried:
        if self.delta.is_account_epheremally_burried(account_key) {
            return self.delta.accounts_to_burry.get(&account_key).cloned();
        }

        // 2 Check in the in-memory burried accounts.
        self.in_memory_burried_accounts.get(&account_key).cloned()
    }

    /// Burries an account to the graveyard.
    ///
    /// NOTE: These changes are saved with the use of the `apply_changes` function.
    pub fn burry_account(
        &mut self,
        account_key: [u8; 32],
        satoshi_redemption_amount: u64,
    ) -> Result<(), GraveyardBurryAccountError> {
        // 1 Check if the account has just been epheremally burried.
        if self.delta.is_account_epheremally_burried(account_key) {
            return Err(
                GraveyardBurryAccountError::AccountIsAlreadyEpheremallyBurried(account_key),
            );
        }

        // 2 Check if the account has already been burried.
        if self.in_memory_burried_accounts.contains_key(&account_key) {
            return Err(
                GraveyardBurryAccountError::AccountISalreadyPermanentlyBurried(account_key),
            );
        }

        // 3 Epheremally burry the account in the delta.
        self.delta
            .epheremally_burry_account(account_key, satoshi_redemption_amount);

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
        // 1 Check if the account has just been epheremally redeemed.
        if self.delta.is_account_epheremally_redeemed(account_key) {
            return Err(
                GraveyardRedeemAccountCoinsError::AccountCoinsHasJustBeenEphemerallyRedeemed(
                    account_key,
                ),
            );
        }

        // 2 Check if the account has just been epheremally burried.
        if self.delta.is_account_epheremally_burried(account_key) {
            return Err(
                GraveyardRedeemAccountCoinsError::ThisAccountHasJustBeenEphemerallyBurried(
                    account_key,
                ),
            );
        }

        // 3 Get the redemption amount for the account.
        let redemption_amount = self.get_redemption_amount(account_key).ok_or(
            GraveyardRedeemAccountCoinsError::RedemptionAmountNotFound(account_key),
        )?;

        // 4 Check if the redemption amount is less than the minimum redemption amount.
        if redemption_amount < MIN_REDEMPTION_AMOUNT {
            return Err(
                GraveyardRedeemAccountCoinsError::RedemptionAmountIsLessThanTheMinimumRedemptionAmount(
                    account_key,
                    redemption_amount,
                    MIN_REDEMPTION_AMOUNT,
                ),
            );
        }

        // 5 Check if the account has just been epheremally redeemed.
        if self.delta.is_account_epheremally_redeemed(account_key) {
            return Err(
                GraveyardRedeemAccountCoinsError::AccountCoinsHasAlreadyBeenEphemerallyRedeemed(
                    account_key,
                ),
            );
        }

        // 6 Epheremally redeem the account coins in the delta.
        self.delta
            .epheremally_redeem_account_coins(account_key, redemption_amount);

        // 7 Return the result.
        Ok(redemption_amount)
    }

    /// Reverts the epheremal changes associated with the last execution.
    pub fn rollback_last(&mut self) {
        self.restore_delta();
    }

    /// Applies the changes to the graveyard.
    ///
    /// This persists the delta to both in-memory and on-disk storage:
    /// - Redemptions: Resets SatoshiRedemptionAmount to zero for redeemed accounts.
    /// - New accounts: Adds accounts from `accounts_to_burry` to the buried accounts.
    pub fn apply_changes(&mut self) -> Result<(), GraveyardApplyChangesError> {
        // 1 Apply redemptions: Reset redemption amounts to zero in-memory and on-disk.
        for (account_key, _) in self.delta.redemptions.iter() {
            // 1.1 Construct the zero redemption amount.
            let zero_redemption_amount: [u8; 8] = 0u64.to_le_bytes();

            // 1.2 On-disk: Reset the redemption amount to zero.
            self.on_disk_burried_accounts
                .insert(account_key, zero_redemption_amount.to_vec())
                .map_err(|e| {
                    GraveyardApplyChangesError::RedemptionAmountResetError(*account_key, e)
                })?;

            // 1.3 In-memory: Reset the redemption amount to zero.
            self.in_memory_burried_accounts.insert(*account_key, 0);
        }

        // 2 Insert new accounts to burry into in-memory and on-disk.
        for (account_key, satoshi_redemption_amount) in self.delta.accounts_to_burry.iter() {
            // 2.1 On-disk: Insert the buried account.
            self.on_disk_burried_accounts
                .insert(
                    account_key,
                    satoshi_redemption_amount.to_le_bytes().to_vec(),
                )
                .map_err(|e| {
                    GraveyardApplyChangesError::BuriedAccountInsertError(*account_key, e)
                })?;

            // 2.2 In-memory: Insert the buried account.
            self.in_memory_burried_accounts
                .insert(*account_key, *satoshi_redemption_amount);
        }

        // 3 Flush the delta.
        self.delta.flush();

        // 4 Return success.
        Ok(())
    }

    /// Clears all epheremal changes from the delta.
    pub fn flush_delta(&mut self) {
        self.delta.flush();
        self.backup_of_delta.flush();
    }
}
