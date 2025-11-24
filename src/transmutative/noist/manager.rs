use super::{
    dkg::directory::{DKGDirectory, DKG_DIRECTORY},
    session::NOISTSessionCtx,
    setup::setup::VSESetup,
};
use crate::{
    inscriptive::lp::dir::LP_DIRECTORY, transmutative::musig::session::MusigSessionCtx,
    transmutative::secp::into::IntoPointByteVec,
};
use secp::Point;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

/// Guarded DKG manager.
#[allow(non_camel_case_types)]
pub type DKG_MANAGER = Arc<Mutex<DKGManager>>;

#[derive(Clone, Serialize, Deserialize)]
pub enum SessionCtxError {
    InvalidDKGDirHeight,
    InvalidDKGNonceHeight,
}

pub struct DKGManager {
    lp_dir: LP_DIRECTORY,
    directories: HashMap<u64, DKG_DIRECTORY>,
    setup_db: sled::Db,
}

impl DKGManager {
    pub fn new(lp_dir: &LP_DIRECTORY) -> Option<DKG_MANAGER> {
        let setup_db = sled::open("storage/noist/setup").ok()?;

        let mut directories = HashMap::<u64, DKG_DIRECTORY>::new();

        for lookup in setup_db.iter() {
            if let Ok((_, setup_)) = lookup {
                let setup: VSESetup = serde_json::from_slice(&setup_).ok()?;
                let setup_height = setup.height();
                let dkg_directory: DKGDirectory = DKGDirectory::new(&setup)?;
                directories.insert(setup_height, Arc::new(Mutex::new(dkg_directory)));
            }
        }

        let manager_ = Some(DKGManager {
            lp_dir: Arc::clone(lp_dir),
            directories,
            setup_db,
        })?;

        Some(Arc::new(Mutex::new(manager_)))
    }

    pub fn lp_directory(&self) -> LP_DIRECTORY {
        Arc::clone(&self.lp_dir)
    }

    pub fn directories(&self) -> HashMap<u64, DKG_DIRECTORY> {
        self.directories.clone()
    }

    pub fn active_directory(&self) -> Option<DKG_DIRECTORY> {
        match self.directories.iter().max_by_key(|(&k, _)| k) {
            Some((_, dir)) => Some(Arc::clone(&dir)),
            None => None,
        }
    }

    pub fn directory_by_height(&self, dir_height: u64) -> Option<DKG_DIRECTORY> {
        Some(Arc::clone(self.directories.get(&dir_height)?))
    }

    pub async fn directory_by_key(&self, group_key: Point) -> Option<DKG_DIRECTORY> {
        for (_, dir) in self.directories.iter() {
            let dir_group_key = {
                let dir_ = dir.lock().await;
                dir_.group_key()
            }?;

            if dir_group_key == group_key {
                return Some(Arc::clone(&dir));
            }
        }
        None
    }

    pub async fn full_operator_list(&self) -> Vec<[u8; 32]> {
        let mut full_list = Vec::<[u8; 32]>::new();

        for (_, dir) in self.directories.iter() {
            let _dkg_dir = dir.lock().await;

            if let Ok(list) = _dkg_dir.setup().signatories().into_xpoint_vec() {
                full_list.extend(list);
            }
        }

        full_list
    }

    pub fn insert_setup(&mut self, setup: &VSESetup) -> bool {
        let setup_height = setup.height();

        if self.directories.contains_key(&setup_height) {
            return false;
        };

        if let Err(_) = self
            .setup_db
            .insert(setup.height().to_be_bytes(), setup.serialize())
        {
            return false;
        }

        let new_directory = match DKGDirectory::new(setup) {
            Some(directory) => directory,
            None => return false,
        };

        if let Some(_) = self
            .directories
            .insert(setup_height, Arc::new(Mutex::new(new_directory)))
        {
            return false;
        }

        true
    }

    pub fn setup_height(&self) -> u64 {
        self.directories.keys().max().unwrap_or(&0).to_owned()
    }

    pub async fn pick_signing_session(
        &self,
        dir_height: u64,
        message: [u8; 32],
        musig_ctx: Option<MusigSessionCtx>,
        toxic: bool,
    ) -> Option<NOISTSessionCtx> {
        let dkg_dir: DKG_DIRECTORY = self.directory_by_height(dir_height)?;
        let mut dkg_dir_ = dkg_dir.lock().await;
        let nonce_height = dkg_dir_.pick_index()?;
        dkg_dir_.signing_session(message, nonce_height, musig_ctx, toxic)
    }

    pub async fn musig_nested_signing_session(
        &self,
        dir_height: u64,
        nonce_height: u64,
        musig_ctx: MusigSessionCtx,
        toxic: bool,
    ) -> Result<NOISTSessionCtx, SessionCtxError> {
        let dkg_dir: DKG_DIRECTORY = match self.directory_by_height(dir_height) {
            Some(dir) => dir,
            None => return Err(SessionCtxError::InvalidDKGDirHeight),
        };

        let mut dkg_dir_ = dkg_dir.lock().await;

        let session = match dkg_dir_.signing_session(
            musig_ctx.message(),
            nonce_height,
            Some(musig_ctx),
            toxic,
        ) {
            Some(session) => session,
            None => return Err(SessionCtxError::InvalidDKGNonceHeight),
        };

        Ok(session)
    }

    pub async fn vanilla_signing_session(
        &self,
        dir_height: u64,
        nonce_height: u64,
        message: [u8; 32],
        toxic: bool,
    ) -> Result<NOISTSessionCtx, SessionCtxError> {
        let dkg_dir: DKG_DIRECTORY = match self.directory_by_height(dir_height) {
            Some(dir) => dir,
            None => return Err(SessionCtxError::InvalidDKGDirHeight),
        };

        let mut dkg_dir_ = dkg_dir.lock().await;

        let session = match dkg_dir_.signing_session(message, nonce_height, None, toxic) {
            Some(session) => session,
            None => return Err(SessionCtxError::InvalidDKGNonceHeight),
        };

        Ok(session)
    }
}
