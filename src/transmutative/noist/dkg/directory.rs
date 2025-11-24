use super::session::DKGSession;
use crate::{
    transmutative::musig::session::MusigSessionCtx,
    transmutative::noist::{session::NOISTSessionCtx, setup::setup::VSESetup},
};
use secp::Point;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

/// Guarded DKG directory.
#[allow(non_camel_case_types)]
pub type DKG_DIRECTORY = Arc<Mutex<DKGDirectory>>;

#[derive(Clone)]
pub struct DKGDirectory {
    setup: VSESetup,                    // VSE setup.
    sessions: HashMap<u64, DKGSession>, // In-memory DKG sessions (index, session).
    sessions_db: sled::Db,              // Database connection.
    nonce_height: u64,
    nonce_height_db: sled::Db,
}
// 'storage/signatory/dkg/batches/BATCH_NO' key:SESSION_NONCE
impl DKGDirectory {
    pub fn new(setup: &VSESetup) -> Option<Self> {
        let setup_height = setup.height();
        // sessions path 'storage/signatory/dkg/batches/BATCH_NO/sessions' key is SESSION_INDEX
        // manager path 'storage/signatory/dkg/batches/manager' key is BATCH_NO
        let mut nonce_height: u64 = 0;

        let nonce_height_path = format!("{}/{}", "storage/noist/dkgdir/", setup_height);
        let nonce_height_db = sled::open(nonce_height_path).ok()?;

        if let Ok(lookup) = nonce_height_db.get(&[0x00]) {
            if let Some(height) = lookup {
                nonce_height = u64::from_be_bytes(height.as_ref().try_into().ok()?);
            }
        };

        let sessions_path = format!("{}/{}/{}", "storage/noist/dkgdir", setup_height, "dkgses");
        let sessions_db = sled::open(sessions_path).ok()?;

        let mut sessions = HashMap::<u64, DKGSession>::new();

        // Insert DKG sessions to the memory.
        for lookup in sessions_db.iter() {
            if let Ok((_, session)) = lookup {
                let session: DKGSession = match serde_json::from_slice(&session) {
                    Ok(session) => session,
                    Err(_) => return None,
                };

                sessions.insert(session.index(), session);
            }
        }

        Some(DKGDirectory {
            setup: setup.to_owned(),
            sessions,
            sessions_db,
            nonce_height,
            nonce_height_db,
        })
    }

    pub fn set_index_height(&mut self, new_height: u64) {
        if self.nonce_height < new_height {
            let _ = self
                .nonce_height_db
                .insert(&[0x00], &new_height.to_be_bytes());
            self.nonce_height = new_height;
        }
    }

    pub fn setup(&self) -> VSESetup {
        self.setup.clone()
    }

    pub fn sessions(&self) -> HashMap<u64, DKGSession> {
        self.sessions.clone()
    }

    pub fn sessions_db(&self) -> sled::Db {
        self.sessions_db.clone()
    }

    pub fn dir_height(&self) -> u64 {
        self.setup.height()
    }

    pub fn nonce_height(&self) -> u64 {
        self.nonce_height
    }

    pub fn signatories(&self) -> Vec<Point> {
        self.setup.signatories()
    }

    pub fn group_key_session(&self) -> Option<DKGSession> {
        Some(self.sessions.get(&0)?.to_owned())
    }

    pub fn group_nonce_session(&self, nonce: u64) -> Option<DKGSession> {
        // Nonce '0' is allocated for the group key.
        if nonce == 0 {
            return None;
        }
        Some(self.sessions.get(&nonce)?.to_owned())
    }

    pub fn group_key(&self) -> Option<Point> {
        let group_key_session = self.group_key_session()?;
        let group_key = group_key_session.group_combined_full_point(None, None)?;
        Some(group_key)
    }

    pub fn group_nonce(&self, nonce_height: u64, message: [u8; 32]) -> Option<Point> {
        let nonce_session = self.group_nonce_session(nonce_height)?;
        let group_key_bytes = self.group_key()?.serialize_xonly();
        let group_nonce =
            nonce_session.group_combined_full_point(Some(group_key_bytes), Some(message))?;
        Some(group_nonce)
    }

    pub fn remove_session(&mut self, height: u64) -> bool {
        // Group key session cannot be removed.
        if height == 0 {
            return false;
        }

        match self.sessions_db.remove(height.to_be_bytes()) {
            Ok(removed) => {
                if let None = removed {
                    return false;
                }
            }
            Err(_) => return false,
        };

        if let None = self.sessions.remove(&height) {
            return false;
        }

        true
    }

    pub fn num_nonce_sessions(&self) -> u64 {
        (self.sessions.len() as u64) - 1 // Total number of sessions minus the key session.
    }

    pub fn new_session_to_fill(&mut self) -> Option<DKGSession> {
        let new_index_height = {
            match self.group_key_session() {
                None => 0,
                Some(_) => {
                    let new_index_height = self.nonce_height + 1;
                    self.set_index_height(new_index_height);
                    new_index_height
                }
            }
        };

        DKGSession::new(new_index_height, &self.signatories())
    }

    pub fn insert_session_filled(&mut self, session: &DKGSession) -> bool {
        // Nonce session insertions are not allowed, if key session is not set.

        let session_index = session.index();

        if let Some(_) = self.sessions.get(&session_index) {
            return false;
        }

        if let None = self.group_key_session() {
            if session_index != 0 {
                return false;
            }
        }

        if !session.verify(&self.setup) {
            return false;
        }

        if let Ok(_) = self
            .sessions_db
            .insert(session.index().to_be_bytes(), session.serialize())
        {
            if let None = self.sessions.insert(session_index, session.to_owned()) {
                self.set_index_height(session_index);
                return true;
            }
        }
        false
    }

    pub fn available_sessions(&self) -> u64 {
        self.sessions.len() as u64
    }

    pub fn pick_index(&self) -> Option<u64> {
        let index = self.sessions.keys().filter(|&&key| key != 0).min()?;
        Some(index.to_owned())
    }

    pub fn pick_signing_session(
        &mut self,
        message: [u8; 32],
        musig_ctx: Option<MusigSessionCtx>,
        toxic: bool,
    ) -> Option<NOISTSessionCtx> {
        let nonce_height = self.pick_index()?;
        self.signing_session(message, nonce_height, musig_ctx, toxic)
    }

    pub fn signing_session(
        &mut self,
        message: [u8; 32],
        nonce_height: u64,
        musig_ctx: Option<MusigSessionCtx>,
        toxic: bool,
    ) -> Option<NOISTSessionCtx> {
        let group_key_session = self.group_key_session()?;

        let group_nonce_session = self.group_nonce_session(nonce_height)?;

        let group_key = self.group_key()?;

        let hiding_group_nonce = group_nonce_session.group_combined_hiding_point()?;
        let post_binding_group_nonce = group_nonce_session
            .group_combined_post_binding_point(Some(group_key.serialize_xonly()), Some(message))?;

        let group_nonce = self.group_nonce(nonce_height, message)?;

        let signing_session = NOISTSessionCtx::new(
            &group_key_session,
            &group_nonce_session,
            group_key,
            hiding_group_nonce,
            post_binding_group_nonce,
            group_nonce,
            message,
            musig_ctx,
        )?;

        if toxic {
            self.remove_session(nonce_height);
        }

        Some(signing_session)
    }
}
