use super::epoch::Epoch;
use crate::{
    constructive::entity::account::account::Account, inscriptive::baked::INITIAL_OPERATOR_SET,
    operative::Chain, transmutative::secp::into::IntoPoint,
};
use secp::Point;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

/// Guarded epoch directory.
#[allow(non_camel_case_types)]
pub type EPOCH_DIRECTORY = Arc<Mutex<EpochDirectory>>;

/// Directory for the operator quorum epoches.
pub struct EpochDirectory {
    // In-memory list.
    epochs: HashMap<u64, Epoch>,
    // In-storage db.
    db: sled::Db,
}

impl EpochDirectory {
    pub fn new(chain: Chain) -> Option<EPOCH_DIRECTORY> {
        let path = format!("{}/{}/{}", "storage", chain.to_string(), "dir/epoch");
        let db = sled::open(path).ok()?;

        let mut epochs = HashMap::<u64, Epoch>::new();

        // Remove later
        {
            let test_group_key = Point::from_hex(
                "032b4e599dcecb2ba1527cb47cb25e2fc704082d5ea6390ccc10183570febdbaa1",
            )
            .ok()
            .unwrap();
            let test_epoch = Epoch::new(1, true, test_group_key, vec![]);
            epochs.insert(1, test_epoch);
        }

        for lookup in db.iter() {
            if let Ok((key, val)) = lookup {
                let height: u64 = u64::from_be_bytes(key.as_ref().try_into().ok()?);
                let epoch: Epoch = serde_json::from_slice(&val).ok()?;

                epochs.insert(height, epoch);
            }
        }

        let epoch_dir = EpochDirectory { epochs, db };

        Some(Arc::new(Mutex::new(epoch_dir)))
    }

    pub fn insert_epoch(&mut self, epoch: &Epoch) -> bool {
        let height = epoch.height();

        // Insert in-memory.
        if let Some(_) = self.epochs.insert(height, epoch.to_owned()) {
            return false;
        }

        // Insert in-db.
        match self.db.insert(height.to_be_bytes(), epoch.serialize()) {
            Ok(_) => return true,
            Err(_) => return false,
        }
    }
    /// Returns the active group keys.
    pub fn active_group_keys(&self) -> Vec<Point> {
        self.epochs
            .iter()
            .filter(|(_, epoch)| epoch.active())
            .map(|(_, epoch)| epoch.group_key())
            .collect()
    }

    /// Returns the current epoch height.
    pub fn current_epoch_height(&self) -> u64 {
        match self.epochs.iter().max_by_key(|(&k, _)| k) {
            Some((&height, _)) => height,
            None => 0,
        }
    }

    /// Returns the next epoch height.
    pub fn next_epoch_height(&self) -> u64 {
        self.current_epoch_height() + 1
    }

    /// Returns the current epoch.
    pub fn current_epoch(&self) -> Option<Epoch> {
        match self.epochs.iter().max_by_key(|(&k, _)| k) {
            Some((_, epoch)) => Some(epoch.to_owned()),
            None => None,
        }
    }

    /// Returns the latest active epoch.
    pub fn latest_active_epoch(&self) -> Option<Epoch> {
        self.epochs
            .iter()
            .filter(|(_, epoch)| epoch.active())
            .max_by_key(|(height, _)| *height)
            .map(|(_, epoch)| epoch.to_owned())
    }

    /// Returns the operator set.       
    pub fn operator_set(&self) -> Vec<Point> {
        let mut operator_set = Vec::<Point>::new();

        // Fill with the initial operator set.
        {
            operator_set.extend(
                INITIAL_OPERATOR_SET
                    .into_iter()
                    .filter_map(|op| op.into_point().ok()),
            );
        }

        // Fill with the epoch operator set.
        {
            operator_set.extend(
                self.epochs
                    .values()
                    .filter(|epoch| epoch.active())
                    .flat_map(|epoch| epoch.operator_keys()),
            );
        }

        operator_set
    }

    /// Returns whether the given account is an operator.
    pub fn is_operator(&self, account: Account) -> bool {
        self.operator_set().contains(&account.key())
    }
}
