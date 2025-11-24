use super::lp::LP;
use crate::{
    constructive::entity::account::account::Account, operative::Chain,
    transmutative::secp::into::IntoPointVec,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Guarded liquidity provider directory.
#[allow(non_camel_case_types)]
pub type LP_DIRECTORY = Arc<Mutex<LPDirectory>>;

/// Directory for the liquidity providers.
pub struct LPDirectory {
    // In-memory list.
    lps: Vec<LP>,
    // In-storage db.
    db: sled::Db,
}

impl LPDirectory {
    pub fn new(chain: Chain) -> Option<LP_DIRECTORY> {
        let path = format!("{}/{}/{}", "storage", chain.to_string(), "dir/lp");
        let db = sled::open(path).ok()?;

        let mut lps = Vec::<LP>::new();

        for lookup in db.iter() {
            if let Ok((_, val)) = lookup {
                let lp: LP = serde_json::from_slice(&val).ok()?;

                lps.push(lp);
            }
        }

        let lp_dir = LPDirectory { lps, db };

        Some(Arc::new(Mutex::new(lp_dir)))
    }

    /// Returns the LP for the given account.
    pub fn lp(&self, account: Account) -> Option<LP> {
        match self.lps.iter().find(|lp| lp.account() == account).cloned() {
            Some(lp) => match lp.empty_liquidity() {
                true => None,
                false => Some(lp),
            },
            None => None,
        }
    }

    /// Returns whether the given account is a liquidity provider.
    pub fn is_lp(&self, account: Account) -> bool {
        match self.lp(account) {
            Some(_) => true,
            None => false,
        }
    }

    /// Adds a new LP if not exists, otherwise updates the existing LP.
    fn update_add(&mut self, lp: &LP) {
        // Update in-memory.
        if let Some(index) = self.lps.iter().position(|lp| lp.account() == lp.account()) {
            self.lps[index] = lp.to_owned();
        } else {
            self.lps.push(lp.to_owned());
        }

        // Update in-storage.
        let _ = self
            .db
            .insert(lp.account().key().serialize_xonly(), lp.serialize());
    }

    /// Checks if the given keys withing the subset of liquidity providers.
    pub fn is_valid_subset(&self, keys: &Vec<[u8; 32]>) -> bool {
        let key_vec = match keys.into_point_vec() {
            Ok(vec) => vec,
            Err(_) => return false,
        };

        for key in key_vec.iter() {
            let account = match Account::new(key.to_owned(), None, None) {
                Some(account) => account,
                None => return false,
            };

            if let None = self.lp(account) {
                return false;
            }
        }

        true
    }

    /// Returns the list of LPs sorted by the liquidity.
    pub fn lp_rank_list(&self, top: u64) -> Vec<LP> {
        let mut ranked_lps = self.lps.clone();
        ranked_lps.sort_by(|a, b| b.lp().cmp(&a.lp()));
        ranked_lps.into_iter().take(top as usize).collect()
    }

    /// Returns the list of LP keys sorted by the liquidity.
    pub fn lp_rank_key_list(&self, top: u64) -> Vec<[u8; 32]> {
        self.lp_rank_list(top)
            .iter()
            .map(|lp| lp.account().key().serialize_xonly())
            .collect()
    }

    /// Updates the liquidity of the given account.
    pub fn liquidity_add(&mut self, account: Account, liquidity_add: u64, lp_add: u64) -> bool {
        let mut lp = self
            .lps
            .iter()
            .find(|lp| lp.account() == account)
            .cloned()
            .unwrap_or_else(|| LP::new(account, 0, 0));

        if !lp.update_add(liquidity_add, lp_add) {
            return false;
        }

        self.update_add(&lp);

        true
    }

    /// Removes the liquidity of the given account.
    pub fn liquidity_remove(
        &mut self,
        account: Account,
        liquidity_remove: u64,
        lp_remove: u64,
    ) -> bool {
        let mut lp = self
            .lps
            .iter()
            .find(|lp| lp.account() == account)
            .cloned()
            .unwrap_or_else(|| LP::new(account, 0, 0));

        if !lp.update_remove(liquidity_remove, lp_remove) {
            return false;
        }

        self.update_add(&lp);

        true
    }
}
