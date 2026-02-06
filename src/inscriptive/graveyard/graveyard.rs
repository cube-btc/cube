use crate::inscriptive::graveyard::delta::delta::GraveyardDelta;
use crate::inscriptive::graveyard::errors::construction_error::GraveyardConstructionError;
use crate::operative::Chain;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Account key.
type AccountKey = [u8; 32];

/// Satoshi redemption amount.
type SatoshiRedemptionAmount = u64;

/// Local storage manager for the storing destroyed accounts.
pub struct Graveyard {
    // In-memory destroyed accounts.
    in_memory_destroyed_accounts: HashMap<AccountKey, SatoshiRedemptionAmount>,

    // On-disk db for storing the destroyed accounts.
    on_disk_destroyed_accounts: sled::Db,

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

        // 2 Initialize the in-memory destroyed accounts.
        let mut in_memory_destroyed_accounts =
            HashMap::<AccountKey, SatoshiRedemptionAmount>::new();

        // 3 Iterate over all items in the graveyard db to collect the destroyed accounts.
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

                // 3.1.3 Insert the destroyed account into the in-memory destroyed accounts.
                in_memory_destroyed_accounts.insert(account_key, satoshi_redemption_amount);
            }
        }

        // 4 Construct the graveyard.
        let graveyard = Graveyard {
            in_memory_destroyed_accounts,
            on_disk_destroyed_accounts: graveyard_db,
            delta: GraveyardDelta::fresh_new(),
            backup_of_delta: GraveyardDelta::fresh_new(),
        };

        // 5 Guard the graveyard.
        let graveyard = Arc::new(Mutex::new(graveyard));

        // 6 Return the graveyard.
        Ok(graveyard)
    }
}
