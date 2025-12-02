use super::{
    commitnack::CSessionCommitNack, opcov::CSessionOpCov, opcovack::OSessionOpCovAck,
    uphold::NSessionUphold, upholdack::CSessionUpholdAck, upholdnack::CSessionUpholdNack,
};
use crate::{
    communicative::{
        peer::{manager::PEER_MANAGER, peer::PEER},
        tcp::client::TCPClient,
    },
    constructive::{
        entity::account::account::Account,
        entry::entry::Entry,
        txo::{
            connector::Connector,
            lift::Lift,
            projector::{Projector, ProjectorTag},
        },
    },
    inscriptive::{
        blacklist::BLIST_DIRECTORY, registery_manager::registery_manager::REGISTERY_MANAGER,
    },
    operative::session::{
        allowance::allowance, commit::NSessionCommit, commitack::CSessionCommitAck,
    },
    transmutative::{
        hash::{Hash, HashTag},
        musig::{keyagg::MusigKeyAggCtx, session::MusigSessionCtx},
        noist::{dkg::directory::DKG_DIRECTORY, manager::DKG_MANAGER, session::NOISTSessionCtx},
        secp::authenticable::{AuthSighash, Authenticable},
    },
};
use async_trait::async_trait;
use colored::Colorize;
use secp::{Point, Scalar};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    sync::Mutex,
    time::{sleep, Instant},
};

/// Guarded coordinator session context.
#[allow(non_camel_case_types)]
pub type CSESSION_CTX = Arc<Mutex<CSessionCtx>>;

type DKGDirHeight = u64;
type DKGNonceHeight = u64;

pub const ON_STAGE_WAIT_TIME_REGULAR: Duration = Duration::from_secs(10);
pub const ON_STAGE_WAIT_TIME_POSTUPHELDERR: Duration = Duration::from_secs(1500);
pub const UPHOLD_TIMEOUT: Duration = Duration::from_millis(1500);

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CSessionStage {
    On,        // The session is on. New commitments are now allowed to join.
    Locked,    // The session is locked. No new commitments are allowed.
    Upheld,    // The session is upheld. All partial MuSig signatures are collected.
    Finalized, // The session is finalized. All forfeiture signatures are collected.
    Off,       // The session is off.
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct CSessionCtx {
    dkg_manager: DKG_MANAGER,
    peer_manager: PEER_MANAGER,
    blacklist_dir: BLIST_DIRECTORY,
    registery: REGISTERY_MANAGER,
    //
    session_id: [u8; 32],
    stage: CSessionStage,
    // Commit pool.
    commit_pool: Vec<NSessionCommit>,
    // Post-commit-pool pruned (failed) commits
    pruned_commits: Vec<NSessionCommit>,
    // Post-commit-pool passed commits
    passed_commits: Vec<NSessionCommit>,
    // Entries
    entries: Vec<Entry>,
    // Payload Auth:
    payload_auth_nonces: HashMap<Account, (Point, Point)>,
    payload_auth_ctxes: Option<(
        DKGDirHeight,
        DKGNonceHeight,
        NOISTSessionCtx,
        MusigSessionCtx,
    )>,
    // VTXO projector:
    vtxo_projector_nonces: HashMap<Account, (Point, Point)>,
    vtxo_projector_ctxes: Option<(
        DKGDirHeight,
        DKGNonceHeight,
        NOISTSessionCtx,
        MusigSessionCtx,
    )>,
    // Connector projector:
    connector_projector_nonces: HashMap<Account, (Point, Point)>,
    connector_projector_ctxes: Option<(
        DKGDirHeight,
        DKGNonceHeight,
        NOISTSessionCtx,
        MusigSessionCtx,
    )>,
    // ZKP contingent:
    zkp_contingent_nonces: HashMap<Account, (Point, Point)>,
    zkp_contingent_ctxes: Option<(
        DKGDirHeight,
        DKGNonceHeight,
        NOISTSessionCtx,
        MusigSessionCtx,
    )>,
    // Lift txos:
    lift_prevtxo_nonces: HashMap<Account, HashMap<Lift, (Point, Point)>>,
    lift_prevtxo_ctxes: HashMap<
        Account,
        HashMap<
            Lift,
            (
                DKGDirHeight,
                DKGNonceHeight,
                NOISTSessionCtx,
                MusigSessionCtx,
            ),
        >,
    >,
    // Connectors:
    connector_txo_nonces: HashMap<Account, Vec<(Point, Point)>>,
    connector_txo_ctxes: HashMap<
        Account,
        Vec<(
            DKGDirHeight,
            DKGNonceHeight,
            NOISTSessionCtx,
            MusigSessionCtx,
        )>,
    >,
}

impl CSessionCtx {
    pub fn construct(
        dkg_manager: &DKG_MANAGER,
        peer_manager: &PEER_MANAGER,
        blacklist_dir: &BLIST_DIRECTORY,
        registery: &REGISTERY_MANAGER,
    ) -> CSESSION_CTX {
        let session = CSessionCtx {
            dkg_manager: Arc::clone(dkg_manager),
            peer_manager: Arc::clone(peer_manager),
            blacklist_dir: Arc::clone(blacklist_dir),
            registery: Arc::clone(registery),
            session_id: [0xffu8; 32],
            stage: CSessionStage::Off,
            commit_pool: Vec::<NSessionCommit>::new(),
            pruned_commits: Vec::<NSessionCommit>::new(),
            // Post-commit-pool valid commits
            passed_commits: Vec::<NSessionCommit>::new(),
            // Entries
            entries: Vec::<Entry>::new(),
            payload_auth_nonces: HashMap::<Account, (Point, Point)>::new(),
            payload_auth_ctxes: None,
            vtxo_projector_nonces: HashMap::<Account, (Point, Point)>::new(),
            vtxo_projector_ctxes: None,
            connector_projector_nonces: HashMap::<Account, (Point, Point)>::new(),
            connector_projector_ctxes: None,
            zkp_contingent_nonces: HashMap::<Account, (Point, Point)>::new(),
            zkp_contingent_ctxes: None,
            lift_prevtxo_nonces: HashMap::<Account, HashMap<Lift, (Point, Point)>>::new(),
            lift_prevtxo_ctxes: HashMap::<
                Account,
                HashMap<
                    Lift,
                    (
                        DKGDirHeight,
                        DKGNonceHeight,
                        NOISTSessionCtx,
                        MusigSessionCtx,
                    ),
                >,
            >::new(),
            connector_txo_nonces: HashMap::<Account, Vec<(Point, Point)>>::new(),
            connector_txo_ctxes: HashMap::<
                Account,
                Vec<(
                    DKGDirHeight,
                    DKGNonceHeight,
                    NOISTSessionCtx,
                    MusigSessionCtx,
                )>,
            >::new(),
        };
        Arc::new(Mutex::new(session))
    }

    fn dkg_manager(&self) -> DKG_MANAGER {
        Arc::clone(&self.dkg_manager)
    }

    fn peer_manager(&self) -> PEER_MANAGER {
        Arc::clone(&self.peer_manager)
    }

    pub fn init(&mut self, session_id: [u8; 32]) {
        self.session_id = session_id;
        self.reset();
        self.on();
    }

    pub fn session_id(&self) -> [u8; 32] {
        self.session_id
    }

    pub fn stage(&self) -> CSessionStage {
        self.stage
    }

    pub fn num_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn accounts(&self) -> Vec<Account> {
        self.entries.iter().map(|entry| entry.account()).collect()
    }

    fn is_account(&self, account: &Account) -> bool {
        self.entries
            .iter()
            .any(|entry| entry.account().key() == account.key())
    }

    fn commit_pool_overlap(&self, account: Account) -> bool {
        self.commit_pool
            .iter()
            .any(|commit| commit.entry().account().key() == account.key())
    }

    /// Sets the coordinator session stage to `on`
    /// meaning accounts can now participate in a rollup state transition.
    pub fn on(&mut self) {
        self.stage = CSessionStage::On;
    }

    /// Validates a given `NSessionCommit` to ensure that the involved account
    /// and transactions are eligible for the rollup state transition.
    async fn validate_commit(
        &self,
        auth_commit: &Authenticable<NSessionCommit>,
    ) -> Result<(), CSessionCommitNack> {
        let dkg_manager: DKG_MANAGER = Arc::clone(&self.dkg_manager);

        if !auth_commit.authenticate() {
            return Err(CSessionCommitNack::AuthErr);
        }

        let commit = auth_commit.object();
        let account = commit.account();

        if auth_commit.key() != account.key().serialize_xonly() {
            return Err(CSessionCommitNack::AuthErr);
        }
        // #1 Blacklist check.
        {
            let _blacklist_dir = self.blacklist_dir.lock().await;
            if let Some(until) = _blacklist_dir.check_blacklist(account) {
                return Err(CSessionCommitNack::BlacklistedUntil(until));
            }
        }

        // #2 Entry-account validation
        if !commit.entry().validate_account() {
            return Err(CSessionCommitNack::AuthErr);
        }

        // #3 Overlap check
        if self.commit_pool_overlap(account) {
            return Err(CSessionCommitNack::Overlap);
        }

        // #4 Allowance check
        if allowance(account) {
            return Err(CSessionCommitNack::Allowance);
        }

        // #5 Lift prevtxouts validation
        for (lift, _) in commit.lift_prevtxo_nonces().iter() {
            // #1 Operator key validation
            {
                let lift_operator_key = lift.operator_key();
                let lift_account_key = lift.account_key();

                if lift_operator_key == lift_account_key {
                    return Err(CSessionCommitNack::InvalidLiftOperatorKey);
                }

                if lift_account_key != account.key() {
                    return Err(CSessionCommitNack::InvalidLiftAccountKey);
                }

                {
                    let dkg_manager_ = dkg_manager.lock().await;
                    if let None = dkg_manager_.directory_by_key(lift_operator_key).await {
                        return Err(CSessionCommitNack::InvalidLiftOperatorKey);
                    }
                }
            }

            // #2 Outpoint validation
            {
                match lift.outpoint() {
                    Some(_outpoint) => {
                        // TODO: check if this is a valid outpoint.
                    }
                    None => return Err(CSessionCommitNack::MissingLiftOutpoint),
                };
            }
        }

        // #6 TODO: Check for num of connectors:
        let _connector_count = self.connector_projector_nonces.len();

        Ok(())
    }

    /// Inserts `NSessionCommit` into the commit pool.
    /// The coordinator awaits `NSessionCommit`s until the `CSessionStage` is set to `on`,  
    /// after which it inserts `NSessionCommit`s into the session context.  
    /// Shortly after, `CSessionStage` is set to `locked`, preventing further insertions for this session.
    pub async fn insert_commit(
        &mut self,
        auth_commit: &Authenticable<NSessionCommit>,
    ) -> Result<(), CSessionCommitNack> {
        // #1 Check session stage
        if self.stage != CSessionStage::On {
            return Err(CSessionCommitNack::SessionLocked);
        }

        // #2 Validate commit
        self.validate_commit(&auth_commit).await?;

        let commit = auth_commit.object();
        let mut account = commit.account();

        // #3 Registery index validation.
        let given_registery_index = account.registery_index();
        let local_registery_index = {
            let _registery_manager = self.registery.lock().await;
            match _registery_manager.get_account_by_key(account.key().serialize_xonly()) {
                Some(account) => account.registery_index(),
                None => None,
            }
        };

        match given_registery_index {
            Some(given_registery_index) => {
                if let Some(local_registery_index) = local_registery_index {
                    if local_registery_index != given_registery_index {
                        return Err(CSessionCommitNack::InvalidAccountRegisteryIndex);
                    }
                }
            }
            None => {
                if let Some(local_registery_index) = local_registery_index {
                    account.set_registery_index(local_registery_index);
                }
            }
        }

        // #4 Insert into commit pool.
        self.commit_pool.push(commit);

        Ok(())
    }

    /// Prunes and order commits within the commit pool.
    async fn order_and_prune_commit_pool(&mut self) {
        // TODO
    }

    /// Check if a commit given account is pruned from commit pool.
    pub fn is_pruned(&self, account: Account) -> bool {
        self.pruned_commits
            .iter()
            .any(|commit| commit.entry().account().key() == account.key())
    }

    /// Sets commits.
    pub async fn set_commits(&mut self) {
        // Filter and order commit pool.
        self.order_and_prune_commit_pool().await;

        // Set commits.
        for commit in self.commit_pool.iter() {
            let account = commit.account();
            let entry = commit.entry();

            // #1 Insert to passed commits.
            self.passed_commits.push(commit.to_owned());

            // #2 Insert to entries.
            self.entries.push(entry);

            // #3 Insert payload auth nonce commitments.
            let payload_auth_nonces = commit.payload_auth_nonces();
            self.payload_auth_nonces
                .insert(account, payload_auth_nonces);

            // #4 Insert vtxo projector nonce commitments.
            let vtxo_projector_nonces = commit.vtxo_projector_nonces();
            self.vtxo_projector_nonces
                .insert(account, vtxo_projector_nonces);

            // #5 Insert connector projector nonce commitments.
            let connector_projector_nonces = commit.connector_projector_nonces();
            self.connector_projector_nonces
                .insert(account, connector_projector_nonces);

            // #6 Insert zkp contingent nonce commitments.
            let zkp_contingent_nonces = commit.zkp_contingent_nonces();
            self.zkp_contingent_nonces
                .insert(account, zkp_contingent_nonces);

            // #7 Insert lift nonce commitments.
            let lift_prevtxo_nonces = commit.lift_prevtxo_nonces();
            self.lift_prevtxo_nonces
                .insert(account, lift_prevtxo_nonces);

            // #8 Insert connector nonce commitments.
            let connector_txo_nonces = commit.connector_txo_nonces();
            self.connector_txo_nonces
                .insert(account, connector_txo_nonces);
        }
    }

    fn payload_auth_msg(&self) -> [u8; 32] {
        let mut preimage = Vec::<u8>::new();

        // Session ID
        preimage.extend(self.session_id);

        // Entries
        for (index, entry) in self.entries.iter().enumerate() {
            let entry_sighash = entry.auth_sighash();
            preimage.extend((index as u32).to_le_bytes());
            preimage.extend(entry_sighash);
        }

        preimage.hash(Some(HashTag::PayloadAuth))
    }

    /// Sets the NOIST and MuSig contexes upon collecting `NSessionCommit`s, triggered by `lock`.
    async fn set_ctxes(&mut self) -> bool {
        // #1 Check session stage
        if self.stage != CSessionStage::Locked {
            return false;
        }

        let dkg_manager: DKG_MANAGER = Arc::clone(&self.dkg_manager);

        let active_dkg_dir: DKG_DIRECTORY = {
            let dkg_manager_ = dkg_manager.lock().await;
            match dkg_manager_.active_directory() {
                Some(dir) => dir,
                None => return false,
            }
        };

        let active_dir_height = {
            let active_dkg_dir_ = active_dkg_dir.lock().await;
            active_dkg_dir_.dir_height()
        };

        let payload_auth_ctxes = match self.payload_auth_nonces.len() {
            0 => return false,
            _ => {
                let payload_auth_message = self.payload_auth_msg();

                let mut dkg_dir_ = active_dkg_dir.lock().await;

                let noist_ctx =
                    match dkg_dir_.pick_signing_session(payload_auth_message, None, true) {
                        Some(session) => session,
                        None => return false,
                    };

                let nonce_height = noist_ctx.nonce_height();
                let mut keys: Vec<Point> = self
                    .payload_auth_nonces
                    .iter()
                    .map(|(account, _)| account.key().clone())
                    .collect();

                keys.push(noist_ctx.group_key());

                let key_agg_ctx = match MusigKeyAggCtx::new(&keys, None) {
                    Some(ctx) => ctx,
                    None => return false,
                };

                let mut musig_ctx = match MusigSessionCtx::new(&key_agg_ctx, payload_auth_message) {
                    Some(ctx) => ctx,
                    None => return false,
                };

                for (msg_sender, (hiding, binding)) in self.payload_auth_nonces.iter() {
                    if !musig_ctx.insert_nonce(
                        msg_sender.key(),
                        hiding.to_owned(),
                        binding.to_owned(),
                    ) {
                        return false;
                    }
                }

                let operator_key = noist_ctx.group_key();
                let operator_hiding = noist_ctx.hiding_group_nonce();
                let operator_binding = noist_ctx.post_binding_group_nonce();

                if !musig_ctx.insert_nonce(operator_key, operator_hiding, operator_binding) {
                    return false;
                }

                Some((active_dir_height, nonce_height, noist_ctx, musig_ctx))
            }
        };

        let vtxo_projector_ctxes = match self.vtxo_projector_nonces.len() {
            0 => None,
            _ => {
                // TODO:
                let vtxo_projector_message = [0xffu8; 32];

                let mut dkg_dir_ = active_dkg_dir.lock().await;

                let noist_ctx =
                    match dkg_dir_.pick_signing_session(vtxo_projector_message, None, true) {
                        Some(session) => session,
                        None => return false,
                    };

                let nonce_height = noist_ctx.nonce_height();

                let remote_keys: Vec<Point> = self
                    .vtxo_projector_nonces
                    .iter()
                    .map(|(account, _)| account.key().clone())
                    .collect();

                let operator_key = noist_ctx.group_key();

                let vtxo_projector =
                    Projector::new(&remote_keys, operator_key, ProjectorTag::VTXOProjector);

                let key_agg_ctx = match vtxo_projector.key_agg_ctx() {
                    Some(ctx) => ctx,
                    None => return false,
                };

                let mut musig_ctx = match MusigSessionCtx::new(&key_agg_ctx, vtxo_projector_message)
                {
                    Some(ctx) => ctx,
                    None => return false,
                };

                for (msg_sender, (hiding, binding)) in self.vtxo_projector_nonces.iter() {
                    if !musig_ctx.insert_nonce(
                        msg_sender.key(),
                        hiding.to_owned(),
                        binding.to_owned(),
                    ) {
                        return false;
                    }
                }

                let operator_key = noist_ctx.group_key();
                let operator_hiding = noist_ctx.hiding_group_nonce();
                let operator_binding = noist_ctx.post_binding_group_nonce();

                if !musig_ctx.insert_nonce(operator_key, operator_hiding, operator_binding) {
                    return false;
                }

                Some((active_dir_height, nonce_height, noist_ctx, musig_ctx))
            }
        };

        let connector_projector_ctxes = match self.connector_projector_nonces.len() {
            0 => None,
            _ => {
                // TODO:
                let connector_projector_message = [0xffu8; 32];

                let mut dkg_dir_ = active_dkg_dir.lock().await;

                let noist_ctx =
                    match dkg_dir_.pick_signing_session(connector_projector_message, None, true) {
                        Some(session) => session,
                        None => return false,
                    };

                let nonce_height = noist_ctx.nonce_height();
                let remote_keys: Vec<Point> = self
                    .connector_projector_nonces
                    .iter()
                    .map(|(account, _)| account.key().clone())
                    .collect();

                let operator_key = noist_ctx.group_key();

                let connector_projector =
                    Projector::new(&remote_keys, operator_key, ProjectorTag::ConnectorProjector);

                let key_agg_ctx = match connector_projector.key_agg_ctx() {
                    Some(ctx) => ctx,
                    None => return false,
                };

                let mut musig_ctx =
                    match MusigSessionCtx::new(&key_agg_ctx, connector_projector_message) {
                        Some(ctx) => ctx,
                        None => return false,
                    };

                for (msg_sender, (hiding, binding)) in self.connector_projector_nonces.iter() {
                    if !musig_ctx.insert_nonce(
                        msg_sender.key(),
                        hiding.to_owned(),
                        binding.to_owned(),
                    ) {
                        return false;
                    }
                }

                let operator_key = noist_ctx.group_key();
                let operator_hiding = noist_ctx.hiding_group_nonce();
                let operator_binding = noist_ctx.post_binding_group_nonce();

                if !musig_ctx.insert_nonce(operator_key, operator_hiding, operator_binding) {
                    return false;
                }

                Some((active_dir_height, nonce_height, noist_ctx, musig_ctx))
            }
        };

        let zkp_contingent_ctxes = match self.zkp_contingent_nonces.len() {
            0 => None,
            _ => {
                // TODO:
                let zkp_contingent_message = [0xffu8; 32];

                let mut dkg_dir_ = active_dkg_dir.lock().await;

                let noist_ctx =
                    match dkg_dir_.pick_signing_session(zkp_contingent_message, None, true) {
                        Some(session) => session,
                        None => return false,
                    };

                let nonce_height = noist_ctx.nonce_height();

                let mut keys: Vec<Point> = self
                    .zkp_contingent_nonces
                    .iter()
                    .map(|(account, _)| account.key().clone())
                    .collect();

                keys.push(noist_ctx.group_key());

                let key_agg_ctx = match MusigKeyAggCtx::new(&keys, None) {
                    Some(ctx) => ctx,
                    None => return false,
                };

                let mut musig_ctx = match MusigSessionCtx::new(&key_agg_ctx, zkp_contingent_message)
                {
                    Some(ctx) => ctx,
                    None => return false,
                };

                for (msg_sender, (hiding, binding)) in self.zkp_contingent_nonces.iter() {
                    if !musig_ctx.insert_nonce(
                        msg_sender.key(),
                        hiding.to_owned(),
                        binding.to_owned(),
                    ) {
                        return false;
                    }
                }

                let operator_key = noist_ctx.group_key();
                let operator_hiding = noist_ctx.hiding_group_nonce();
                let operator_binding = noist_ctx.post_binding_group_nonce();

                if !musig_ctx.insert_nonce(operator_key, operator_hiding, operator_binding) {
                    return false;
                }

                Some((active_dir_height, nonce_height, noist_ctx, musig_ctx))
            }
        };

        // Lift prevtxos:

        let mut lift_prevtxo_ctxes = HashMap::<
            Account,
            HashMap<
                Lift,
                (
                    DKGDirHeight,
                    DKGNonceHeight,
                    NOISTSessionCtx,
                    MusigSessionCtx,
                ),
            >,
        >::new();

        for (msg_sender, lift_prevtxos) in self.lift_prevtxo_nonces.iter() {
            let mut ctxes = HashMap::<
                Lift,
                (
                    DKGDirHeight,
                    DKGNonceHeight,
                    NOISTSessionCtx,
                    MusigSessionCtx,
                ),
            >::new();

            for (lift, (hiding, binding)) in lift_prevtxos.iter() {
                let operator_key = lift.operator_key();

                let (dir_height, nonce_index, noist_ctx, musig_ctx) = {
                    // TODO:
                    let lift_prevtxo_message = [0xffu8; 32];

                    let dkg_dir: DKG_DIRECTORY = {
                        let dkg_manager_ = dkg_manager.lock().await;
                        match dkg_manager_.directory_by_key(operator_key).await {
                            Some(manager) => manager,
                            None => return false,
                        }
                    };

                    let dir_height = {
                        let dkg_dir_ = dkg_dir.lock().await;
                        dkg_dir_.dir_height()
                    };

                    let mut dkg_dir_ = dkg_dir.lock().await;

                    let noist_ctx =
                        match dkg_dir_.pick_signing_session(lift_prevtxo_message, None, true) {
                            Some(session) => session,
                            None => return false,
                        };

                    let nonce_height = noist_ctx.nonce_height();
                    let remote_key = msg_sender.key();

                    let key_agg_ctx = match lift.key_agg_ctx() {
                        Some(ctx) => ctx,
                        None => return false,
                    };

                    let mut musig_ctx =
                        match MusigSessionCtx::new(&key_agg_ctx, lift_prevtxo_message) {
                            Some(ctx) => ctx,
                            None => return false,
                        };

                    if !musig_ctx.insert_nonce(remote_key, hiding.to_owned(), binding.to_owned()) {
                        return false;
                    }

                    let operator_key = noist_ctx.group_key();
                    let operator_hiding = noist_ctx.hiding_group_nonce();
                    let operator_binding = noist_ctx.post_binding_group_nonce();

                    if !musig_ctx.insert_nonce(operator_key, operator_hiding, operator_binding) {
                        return false;
                    }

                    (dir_height, nonce_height, noist_ctx, musig_ctx)
                };
                if let Some(_) = ctxes.insert(
                    lift.to_owned(),
                    (dir_height, nonce_index, noist_ctx, musig_ctx),
                ) {
                    return false;
                }
            }

            if let Some(_) = lift_prevtxo_ctxes.insert(msg_sender.to_owned(), ctxes) {
                return false;
            }
        }

        // Connector TXOs:
        let mut connector_txo_ctxes = HashMap::<
            Account,
            Vec<(
                DKGDirHeight,
                DKGNonceHeight,
                NOISTSessionCtx,
                MusigSessionCtx,
            )>,
        >::new();

        for (msg_sender, connector_txos) in self.connector_txo_nonces.iter() {
            let mut ctxes = Vec::<(
                DKGDirHeight,
                DKGNonceHeight,
                NOISTSessionCtx,
                MusigSessionCtx,
            )>::new();

            for (hiding, binding) in connector_txos.iter() {
                let (dir_height, nonce_height, noist_ctx, musig_ctx) = {
                    // TODO:
                    let connector_txo_message = [0xffu8; 32];

                    let mut dkg_dir_ = active_dkg_dir.lock().await;

                    let noist_ctx =
                        match dkg_dir_.pick_signing_session(connector_txo_message, None, true) {
                            Some(session) => session,
                            None => return false,
                        };

                    let nonce_height = noist_ctx.nonce_height();
                    let remote_key = msg_sender.key();
                    let operator_key = noist_ctx.group_key();

                    let connector = Connector::new(remote_key, operator_key);

                    let key_agg_ctx = match connector.key_agg_ctx() {
                        Some(ctx) => ctx,
                        None => return false,
                    };

                    let mut musig_ctx =
                        match MusigSessionCtx::new(&key_agg_ctx, connector_txo_message) {
                            Some(ctx) => ctx,
                            None => return false,
                        };

                    if !musig_ctx.insert_nonce(remote_key, hiding.to_owned(), binding.to_owned()) {
                        return false;
                    }

                    let operator_key = noist_ctx.group_key();
                    let operator_hiding = noist_ctx.hiding_group_nonce();
                    let operator_binding = noist_ctx.post_binding_group_nonce();

                    if !musig_ctx.insert_nonce(operator_key, operator_hiding, operator_binding) {
                        return false;
                    }

                    (active_dir_height, nonce_height, noist_ctx, musig_ctx)
                };

                ctxes.push((dir_height, nonce_height, noist_ctx, musig_ctx));
            }

            if let Some(_) = connector_txo_ctxes.insert(msg_sender.to_owned(), ctxes) {
                return false;
            }
        }

        // Set values:
        self.payload_auth_ctxes = payload_auth_ctxes;
        self.vtxo_projector_ctxes = vtxo_projector_ctxes;
        self.connector_projector_ctxes = connector_projector_ctxes;
        self.zkp_contingent_ctxes = zkp_contingent_ctxes;
        self.lift_prevtxo_ctxes = lift_prevtxo_ctxes;
        self.connector_txo_ctxes = connector_txo_ctxes;

        true
    }

    /// Locks the session, preventing further `NSessionCommit`s from joining this session.  
    /// New `NSessionCommit`s are put on hold until the `CSessionStage` is set back to `on`.
    pub fn locked(&mut self) {
        self.stage = CSessionStage::Locked;
    }

    /// Returns the respective `CSessionOpCov`s for the DKG operators.
    /// The coordinator does this immediately after locking the session (simultaneously with commitack).
    /// `CSessionOpCov`s contain the post-round-one MuSig contexts to be filled with individual partial signatures (just like `CSessionCommitAck`).
    pub fn opcov(&self) -> Option<CSessionOpCov> {
        // Allowed only in the locked stage
        if self.stage != CSessionStage::Locked {
            return None;
        }

        let entries = self.entries.clone();

        let payload_auth_musig_ctx = match &self.payload_auth_ctxes {
            Some((dir_height, nonce_height, _, musig_ctx)) => (
                dir_height.to_owned(),
                nonce_height.to_owned(),
                musig_ctx.to_owned(),
            ),
            None => return None,
        };

        let vtxo_projector_musig_ctx = match &self.vtxo_projector_ctxes {
            Some((dir_height, nonce_height, _, musig_ctx)) => Some((
                dir_height.to_owned(),
                nonce_height.to_owned(),
                musig_ctx.to_owned(),
            )),
            None => return None,
        };

        let connector_projector_musig_ctx = match &self.connector_projector_ctxes {
            Some((dir_height, nonce_height, _, musig_ctx)) => Some((
                dir_height.to_owned(),
                nonce_height.to_owned(),
                musig_ctx.to_owned(),
            )),
            None => return None,
        };

        let zkp_contingent_musig_ctx = match &self.zkp_contingent_ctxes {
            Some((dir_height, nonce_height, _, musig_ctx)) => Some((
                dir_height.to_owned(),
                nonce_height.to_owned(),
                musig_ctx.to_owned(),
            )),
            None => return None,
        };

        let lift_prevtxo_ctxes_opcov = self
            .lift_prevtxo_ctxes
            .clone()
            .into_iter()
            .map(|(account, inner_map)| {
                let new_inner_map = inner_map
                    .into_iter()
                    .map(|(lift, (dir_height, nonce_height, _, musig_ctx))| {
                        (lift, (dir_height, nonce_height, musig_ctx))
                    })
                    .collect();
                (account, new_inner_map)
            })
            .collect();

        let connector_txo_musig_ctxes_opcov = self
            .connector_txo_ctxes
            .clone()
            .into_iter()
            .map(|(account, vec)| {
                let new_vec = vec
                    .into_iter()
                    .map(|(dir_height, nonce_height, _, musig_ctx)| {
                        (dir_height, nonce_height, musig_ctx)
                    })
                    .collect();
                (account, new_vec)
            })
            .collect();

        let opcov = CSessionOpCov::new(
            entries,
            payload_auth_musig_ctx,
            vtxo_projector_musig_ctx,
            connector_projector_musig_ctx,
            zkp_contingent_musig_ctx,
            lift_prevtxo_ctxes_opcov,
            connector_txo_musig_ctxes_opcov,
        );

        Some(opcov)
    }

    /// Returns the respective `CSessionCommitAck`s for accounts that have previously committed.  
    /// The coordinator does this immediately after locking the session.
    /// `CSessionCommitAck`s contain the post-round-one MuSig contexts to be filled with individual partial signatures.
    /// Those individual partial signatures are returned as part of the subsequent `NSessionUphold`s.
    pub fn commitack(&self, account: Account) -> Result<CSessionCommitAck, CSessionCommitNack> {
        // #1 Check if the session stage is locked.
        if self.stage != CSessionStage::Locked {
            return Err(CSessionCommitNack::SessionNotLocked);
        }

        // #2 Check if the entry is pruned.
        if self.is_pruned(account) {
            return Err(CSessionCommitNack::CommitPruned);
        }

        // #3 Check if account is valid.
        if !self.is_account(&account) {
            return Err(CSessionCommitNack::AccountMismatch);
        }

        let entries = self.entries.clone();

        let payload_auth_musig_ctx = match &self.payload_auth_ctxes {
            Some((_, _, _, ctx)) => ctx.to_owned(),
            None => return Err(CSessionCommitNack::PayloadAuthCtxErr),
        };

        let vtxo_projector_musig_ctx = match &self.vtxo_projector_ctxes {
            Some((_, _, _, ctx)) => Some(ctx.to_owned()),
            None => None,
        };

        let connector_projector_musig_ctx = match &self.connector_projector_ctxes {
            Some((_, _, _, ctx)) => Some(ctx.to_owned()),
            None => None,
        };

        let zkp_contingent_musig_ctx = match &self.zkp_contingent_ctxes {
            Some((_, _, _, ctx)) => Some(ctx.to_owned()),
            None => None,
        };

        let mut lift_prevtxo_musig_ctxes = HashMap::<Lift, MusigSessionCtx>::new();

        for (account, lift_map) in self.lift_prevtxo_ctxes.iter() {
            if account.key() == account.key() {
                for (lift, (_, _, _, ctx)) in lift_map.iter() {
                    lift_prevtxo_musig_ctxes.insert(lift.to_owned(), ctx.to_owned());
                }
            }
        }

        // TODO: prune connectors
        let mut connector_txo_musig_ctxes = Vec::<MusigSessionCtx>::new();

        for (account, ctxes) in self.connector_txo_ctxes.iter() {
            if account.key() == account.key() {
                for (_, _, _, ctx) in ctxes {
                    connector_txo_musig_ctxes.push(ctx.to_owned());
                }
            }
        }

        let session_id = self.session_id;

        let commitack = CSessionCommitAck::new(
            account,
            session_id,
            entries,
            payload_auth_musig_ctx,
            vtxo_projector_musig_ctx,
            connector_projector_musig_ctx,
            zkp_contingent_musig_ctx,
            lift_prevtxo_musig_ctxes,
            connector_txo_musig_ctxes,
        );

        Ok(commitack)
    }

    pub fn insert_opcovack(&mut self, opcovack: OSessionOpCovAck) -> bool {
        // Allowed only in the locked stage
        if self.stage != CSessionStage::Locked {
            return false;
        }

        let signatory = opcovack.signatory();

        // Payload auth
        if let Some((_, _, noist_ctx, _)) = &mut self.payload_auth_ctxes {
            if let Some(partial_sig) = opcovack.payload_auth_partial_sig() {
                if !noist_ctx.insert_partial_sig(signatory, partial_sig) {
                    return false;
                }
            }
        }

        // VTXO projector
        if let Some((_, _, noist_ctx, _)) = &mut self.vtxo_projector_ctxes {
            if let Some(partial_sig) = opcovack.vtxo_projector_partial_sig() {
                if !noist_ctx.insert_partial_sig(signatory, partial_sig) {
                    return false;
                }
            }
        }

        // Connector projector
        if let Some((_, _, noist_ctx, _)) = &mut self.connector_projector_ctxes {
            if let Some(partial_sig) = opcovack.connector_projector_partial_sig() {
                if !noist_ctx.insert_partial_sig(signatory, partial_sig) {
                    return false;
                }
            }
        }

        // zkp contingent
        if let Some((_, _, noist_ctx, _)) = &mut self.zkp_contingent_ctxes {
            if let Some(partial_sig) = opcovack.zkp_contingent_partial_sig() {
                if !noist_ctx.insert_partial_sig(signatory, partial_sig) {
                    return false;
                }
            }
        }

        // Lift prevtxo
        let opcovack_lift_prevtxo_partial_sigs = opcovack.lift_prevtxo_partial_sigs();
        for (account, ctxes) in self.lift_prevtxo_ctxes.iter_mut() {
            let account_partial_sigs = match opcovack_lift_prevtxo_partial_sigs.get(account) {
                Some(sigs) => sigs,
                None => return false,
            };

            for (lift, (_, _, noist_ctx, _)) in ctxes.iter_mut() {
                let partial_sig = match account_partial_sigs.get(lift) {
                    Some(sig) => sig,
                    None => return false,
                };

                if let Some(sig) = partial_sig {
                    if !noist_ctx.insert_partial_sig(opcovack.signatory(), sig.to_owned()) {
                        return false;
                    }
                }
            }
        }

        // Connector txo
        let opcovack_connector_txo_partial_sigs = opcovack.connector_txo_partial_sigs();
        for (account, ctxes) in self.connector_txo_ctxes.iter_mut() {
            let account_partial_sigs = match opcovack_connector_txo_partial_sigs.get(account) {
                Some(sigs) => sigs,
                None => return false,
            };

            for (index, (_, _, noist_ctx, _)) in ctxes.iter_mut().enumerate() {
                let partial_sig = match account_partial_sigs.get(index) {
                    Some(sig) => sig,
                    None => return false,
                };

                if let Some(sig) = partial_sig {
                    if !noist_ctx.insert_partial_sig(opcovack.signatory(), sig.to_owned()) {
                        return false;
                    }
                }
            }
        }

        true
    }

    pub fn opcov_ready(&self) -> bool {
        // Check payload auth
        if let Some((_, _, noist_ctx, _)) = &self.payload_auth_ctxes {
            if let None = noist_ctx.aggregated_sig() {
                return false;
            }
        }

        // Check vtxo projector
        if let Some((_, _, noist_ctx, _)) = &self.vtxo_projector_ctxes {
            if let None = noist_ctx.aggregated_sig() {
                return false;
            }
        }

        // Check connector projector
        if let Some((_, _, noist_ctx, _)) = &self.connector_projector_ctxes {
            if let None = noist_ctx.aggregated_sig() {
                return false;
            }
        }

        // Check zkp contingent
        if let Some((_, _, noist_ctx, _)) = &self.zkp_contingent_ctxes {
            if let None = noist_ctx.aggregated_sig() {
                return false;
            }
        }

        // Check lifts
        for (_, ctxes) in self.lift_prevtxo_ctxes.iter() {
            for (_, (_, _, noist_ctx, _)) in ctxes.iter() {
                if let None = noist_ctx.aggregated_sig() {
                    return false;
                }
            }
        }

        // Check connectors
        for (_, ctxes) in self.connector_txo_ctxes.iter() {
            for (_, _, noist_ctx, _) in ctxes.iter() {
                if let None = noist_ctx.aggregated_sig() {
                    return false;
                }
            }
        }

        true
    }

    pub fn set_operator_agg_sigs(&mut self) -> bool {
        // Payload auth
        if let Some((_, _, noist_ctx, musig_ctx)) = &mut self.payload_auth_ctxes {
            if let Some(agg_sig) = noist_ctx.aggregated_sig() {
                if !musig_ctx.insert_partial_sig(noist_ctx.group_key(), agg_sig) {
                    return false;
                }
            }
        }

        // VTXO projector
        if let Some((_, _, noist_ctx, musig_ctx)) = &mut self.vtxo_projector_ctxes {
            if let Some(agg_sig) = noist_ctx.aggregated_sig() {
                if !musig_ctx.insert_partial_sig(noist_ctx.group_key(), agg_sig) {
                    return false;
                }
            }
        }

        // Connector projector
        if let Some((_, _, noist_ctx, musig_ctx)) = &mut self.connector_projector_ctxes {
            if let Some(agg_sig) = noist_ctx.aggregated_sig() {
                if !musig_ctx.insert_partial_sig(noist_ctx.group_key(), agg_sig) {
                    return false;
                }
            }
        }

        // ZKP contingent
        if let Some((_, _, noist_ctx, musig_ctx)) = &mut self.zkp_contingent_ctxes {
            if let Some(agg_sig) = noist_ctx.aggregated_sig() {
                if !musig_ctx.insert_partial_sig(noist_ctx.group_key(), agg_sig) {
                    return false;
                }
            }
        }

        // Lifts
        for (_, ctxes) in self.lift_prevtxo_ctxes.iter_mut() {
            for (_, (_, _, noist_ctx, musig_ctx)) in ctxes.iter_mut() {
                if let Some(agg_sig) = noist_ctx.aggregated_sig() {
                    if !musig_ctx.insert_partial_sig(noist_ctx.group_key(), agg_sig) {
                        return false;
                    }
                }
            }
        }

        // Connectors
        for (_, ctxes) in self.connector_txo_ctxes.iter_mut() {
            for (_, _, noist_ctx, musig_ctx) in ctxes.iter_mut() {
                if let Some(agg_sig) = noist_ctx.aggregated_sig() {
                    if !musig_ctx.insert_partial_sig(noist_ctx.group_key(), agg_sig) {
                        return false;
                    }
                }
            }
        }

        true
    }

    fn validate_uphold(&self, auth_uphold: &Authenticable<NSessionUphold>) -> bool {
        let account = auth_uphold.object().msg_sender();

        if auth_uphold.key() != account.key().serialize_xonly() {
            return false;
        }

        if !auth_uphold.authenticate() {
            return false;
        }

        if !self.is_account(&account) {
            return false;
        }

        // TODO: additional validations..

        true
    }

    /// Attempts to insert `NSessionUphold` into the session,
    /// which contains partial signatures for the MuSig contexts
    /// requested as part of the earlier `CSessionCommitAck`.
    pub fn insert_uphold(
        &mut self,
        auth_uphold: Authenticable<NSessionUphold>,
    ) -> Result<(), CSessionUpholdNack> {
        // Check if the session is locked.
        if self.stage != CSessionStage::Locked {
            return Err(CSessionUpholdNack::SessionNotLocked);
        }

        // #1 Validate uphold
        if !self.validate_uphold(&auth_uphold) {
            return Err(CSessionUpholdNack::AuthErr);
        }

        let uphold = auth_uphold.object();
        let msg_sender = uphold.msg_sender().key();

        // Insert payload auth partial sig
        if let Some((_, _, _, ctx)) = &mut self.payload_auth_ctxes {
            if !ctx.insert_partial_sig(msg_sender, uphold.payload_auth_partial_sig()) {
                return Err(CSessionUpholdNack::InvalidPayloadAuthSig);
            }
        }

        // Insert VTXO projector partial sig
        if let Some((_, _, _, ctx)) = &mut self.vtxo_projector_ctxes {
            let vtxo_projector_partial_sig = match uphold.vtxo_projector_partial_sig() {
                Some(sig) => sig,
                None => {
                    return Err(CSessionUpholdNack::MissingVTXOProjectorSig);
                }
            };

            if !ctx.insert_partial_sig(msg_sender, vtxo_projector_partial_sig) {
                return Err(CSessionUpholdNack::InvalidVTXOProjectorSig);
            }
        }

        // Insert connector projector partial sig
        if let Some((_, _, _, ctx)) = &mut self.connector_projector_ctxes {
            let connector_projector_partial_sig = match uphold.connector_projector_partial_sig() {
                Some(sig) => sig,
                None => {
                    return Err(CSessionUpholdNack::MissingConnectorProjectorSig);
                }
            };

            if !ctx.insert_partial_sig(msg_sender, connector_projector_partial_sig) {
                return Err(CSessionUpholdNack::InvalidConnectorProjectorSig);
            }
        }

        // Insert ZKP contingent partial sig
        if let Some((_, _, _, ctx)) = &mut self.zkp_contingent_ctxes {
            let zkp_contingent_partial_sig = match uphold.zkp_contingent_partial_sig() {
                Some(sig) => sig,
                None => {
                    return Err(CSessionUpholdNack::MissingZKPContigentSig);
                }
            };

            if !ctx.insert_partial_sig(msg_sender, zkp_contingent_partial_sig) {
                return Err(CSessionUpholdNack::InvalidZKPContigentSig);
            }
        }

        // Insert lift prevtxo partial sigs
        let uphold_lift_prevtxo_partial_sigs = uphold.lift_prevtxo_partial_sigs();
        if let Some(musig_ctxes) = self.lift_prevtxo_ctxes.get_mut(&uphold.msg_sender()) {
            for (lift, (_, _, _, musig_ctx)) in musig_ctxes.iter_mut() {
                let partial_sig = match (&uphold_lift_prevtxo_partial_sigs).get(lift) {
                    Some(sig) => sig,
                    None => {
                        return Err(CSessionUpholdNack::MissingLiftSig);
                    }
                };

                if !musig_ctx.insert_partial_sig(msg_sender, partial_sig.to_owned()) {
                    return Err(CSessionUpholdNack::InvalidLiftSig);
                }
            }
        }

        // Insert connector txo partial sigs
        let uphold_connector_txo_partial_sigs = uphold.connector_txo_partial_sigs();
        if let Some(musig_ctxes) = self.connector_txo_ctxes.get_mut(&uphold.msg_sender()) {
            for (index, (_, _, _, musig_ctx)) in musig_ctxes.iter_mut().enumerate() {
                let partial_sig = match (&uphold_connector_txo_partial_sigs).get(index) {
                    Some(sig) => sig,
                    None => {
                        return Err(CSessionUpholdNack::MissingConnectorSig);
                    }
                };

                if !musig_ctx.insert_partial_sig(msg_sender, partial_sig.to_owned()) {
                    return Err(CSessionUpholdNack::InvalidConnectorSig);
                }
            }
        }

        Ok(())
    }

    /// Returns a lift of accounts who have failed to uphold their commitments.
    pub fn blame_list(&self) -> Vec<Account> {
        let mut blame_list = Vec::<Account>::new();

        // Payload auth scanning
        if let Some((_, _, _, musig_ctx)) = &self.payload_auth_ctxes {
            let musig_blame_list = musig_ctx.blame_list();

            for account in self.accounts().iter() {
                if musig_blame_list.contains(&account.key()) {
                    if !blame_list.contains(account) {
                        blame_list.push(account.to_owned());
                    }
                }
            }
        }

        // VTXO projector scanning
        if let Some((_, _, _, musig_ctx)) = &self.vtxo_projector_ctxes {
            let musig_blame_list = musig_ctx.blame_list();

            for account in self.accounts().iter() {
                if musig_blame_list.contains(&account.key()) {
                    if !blame_list.contains(account) {
                        blame_list.push(account.to_owned());
                    }
                }
            }
        }

        // Connector projector scanning
        if let Some((_, _, _, musig_ctx)) = &self.connector_projector_ctxes {
            let musig_blame_list = musig_ctx.blame_list();

            for account in self.accounts().iter() {
                if musig_blame_list.contains(&account.key()) {
                    if !blame_list.contains(account) {
                        blame_list.push(account.to_owned());
                    }
                }
            }
        }

        // ZKP contingent scanning
        if let Some((_, _, _, musig_ctx)) = &self.zkp_contingent_ctxes {
            let musig_blame_list = musig_ctx.blame_list();

            for account in self.accounts().iter() {
                if musig_blame_list.contains(&account.key()) {
                    if !blame_list.contains(account) {
                        blame_list.push(account.to_owned());
                    }
                }
            }
        }

        // Lifts scanning
        for (account, musig_ctxes) in self.lift_prevtxo_ctxes.iter() {
            for (_, (_, _, _, musig_ctx)) in musig_ctxes {
                let musig_blame_list = musig_ctx.blame_list();

                if musig_blame_list.contains(&account.key()) {
                    if !blame_list.contains(account) {
                        blame_list.push(account.to_owned());
                    }
                }
            }
        }

        // Connectors scanning
        for (account, musig_ctxes) in self.connector_txo_ctxes.iter() {
            for (_, _, _, musig_ctx) in musig_ctxes {
                let musig_blame_list = musig_ctx.blame_list();

                if musig_blame_list.contains(&account.key()) {
                    if !blame_list.contains(account) {
                        blame_list.push(account.to_owned());
                    }
                }
            }
        }

        blame_list
    }

    /// Applies blaming to the blame list.
    pub async fn blame(&self) {
        let blame_list = self.blame_list();

        let mut _blacklist_dir = self.blacklist_dir.lock().await;

        for account in blame_list {
            _blacklist_dir.blame(account);
        }
    }

    fn print_blame_list(&self) {
        let blame_list = self.blame_list();

        if blame_list.len() > 0 {
            let json = serde_json::to_string(&blame_list).unwrap();
            println!("Blame list:\n{}", json);
        }
    }

    pub fn is_upheld_ready(&self) -> bool {
        if self.blame_list().len() > 0 {
            return false;
        }

        if !self.opcov_ready() {
            return false;
        }

        true
    }

    pub fn upholdack(&self, msg_sender: Account) -> Result<CSessionUpholdAck, CSessionUpholdNack> {
        // Check msg.senders blame list.
        let blame_list = self.blame_list();

        if blame_list.len() > 0 {
            return Err(CSessionUpholdNack::BlameMsgSenders(blame_list));
        }

        // Check operator blame.
        if !self.opcov_ready() {
            return Err(CSessionUpholdNack::BlameOperator);
        }

        let payload_auth_agg_sig = self
            .payload_auth_ctxes
            .clone()
            .ok_or(CSessionUpholdNack::PayloadAuthSigErr)?
            .3
            .agg_sig()
            .ok_or(CSessionUpholdNack::PayloadAuthSigErr)?;

        let vtxo_projector_agg_sig = match &self.vtxo_projector_ctxes {
            Some((_, _, _, musig_ctx)) => match musig_ctx.agg_sig() {
                Some(sig) => Some(sig),
                None => return Err(CSessionUpholdNack::VtxoProjectorSigErr),
            },
            None => None,
        };

        let connector_projector_agg_sig = match &self.connector_projector_ctxes {
            Some((_, _, _, musig_ctx)) => match musig_ctx.agg_sig() {
                Some(sig) => Some(sig),
                None => return Err(CSessionUpholdNack::ConnectorProjectorSigErr),
            },
            None => None,
        };

        let zkp_contingent_agg_sig = match &self.zkp_contingent_ctxes {
            Some((_, _, _, musig_ctx)) => match musig_ctx.agg_sig() {
                Some(sig) => Some(sig),
                None => return Err(CSessionUpholdNack::ZkpContigentSigErr),
            },
            None => None,
        };

        // Lift prevtxo agg sigs
        let mut lift_prevtxo_agg_sigs = HashMap::<Lift, Scalar>::new();
        for (account, ctxes) in self.lift_prevtxo_ctxes.iter() {
            if account.key() == msg_sender.key() {
                for (lift, (_, _, _, musig_ctx)) in ctxes.iter() {
                    let agg_sig = match musig_ctx.agg_sig() {
                        Some(sig) => sig,
                        None => return Err(CSessionUpholdNack::LiftSigErr),
                    };

                    lift_prevtxo_agg_sigs.insert(lift.to_owned(), agg_sig);
                }
            }
        }

        // Connector txo agg sigs
        let mut connector_agg_sigs = Vec::<Scalar>::new();
        for (account, ctxes) in self.connector_txo_ctxes.iter() {
            if account.key() == msg_sender.key() {
                for (_, _, _, musig_ctx) in ctxes.iter() {
                    let agg_sig = match musig_ctx.agg_sig() {
                        Some(sig) => sig,
                        None => return Err(CSessionUpholdNack::ConnectorSigErr),
                    };

                    connector_agg_sigs.push(agg_sig);
                }
            }
        }

        let uphold_ack = CSessionUpholdAck::new(
            msg_sender,
            payload_auth_agg_sig,
            vtxo_projector_agg_sig,
            connector_projector_agg_sig,
            zkp_contingent_agg_sig,
            lift_prevtxo_agg_sigs,
            connector_agg_sigs,
        );

        Ok(uphold_ack)
    }

    pub fn upheld(&mut self) {
        self.stage = CSessionStage::Upheld;
    }

    pub fn finalized(&mut self) {
        self.stage = CSessionStage::Finalized;
    }

    pub fn off(&mut self) {
        self.stage = CSessionStage::Off;
    }

    pub fn reset(&mut self) {
        self.stage = CSessionStage::Off;
        self.commit_pool = Vec::<NSessionCommit>::new();
        self.pruned_commits = Vec::<NSessionCommit>::new();
        self.passed_commits = Vec::<NSessionCommit>::new();
        self.entries = Vec::<Entry>::new();
        self.payload_auth_nonces = HashMap::<Account, (Point, Point)>::new();
        self.payload_auth_ctxes = None;
        self.vtxo_projector_nonces = HashMap::<Account, (Point, Point)>::new();
        self.vtxo_projector_ctxes = None;
        self.connector_projector_nonces = HashMap::<Account, (Point, Point)>::new();
        self.connector_projector_ctxes = None;
        self.zkp_contingent_nonces = HashMap::<Account, (Point, Point)>::new();
        self.zkp_contingent_ctxes = None;
        self.lift_prevtxo_nonces = HashMap::<Account, HashMap<Lift, (Point, Point)>>::new();
        self.lift_prevtxo_ctxes = HashMap::<
            Account,
            HashMap<
                Lift,
                (
                    DKGDirHeight,
                    DKGNonceHeight,
                    NOISTSessionCtx,
                    MusigSessionCtx,
                ),
            >,
        >::new();
        self.connector_txo_nonces = HashMap::<Account, Vec<(Point, Point)>>::new();
        self.connector_txo_ctxes = HashMap::<
            Account,
            Vec<(
                DKGDirHeight,
                DKGNonceHeight,
                NOISTSessionCtx,
                MusigSessionCtx,
            )>,
        >::new();
    }
}

#[async_trait]

pub trait CContextRunner {
    /// Handles the background task that runs Cube sessions for the coordinator.
    /// This should be reviewed in line with tcp::server::handle_commit_session and tcp::server::handle_cuphold_session.
    async fn run(&self);
    /// Returns true if all upholds are collected; otherwise, false after a timeout.
    async fn await_upheld(&self) -> bool;
    /// Returns the opcov peer list.
    async fn opcov_peer_list(&self) -> Option<Vec<PEER>>;
    /// Requests `CSessionOpCov`s and retrieves & inserts `OSessionOpCovAck`s.
    async fn opcov_task(&self) -> bool;
}

#[async_trait]
impl CContextRunner for CSESSION_CTX {
    async fn run(&self) {
        let mut waiting_window = ON_STAGE_WAIT_TIME_REGULAR;

        loop {
            // Initialize session
            {
                // TODO:
                let session_id = [0x00u8; 32];

                let mut _session_ctx = self.lock().await;
                _session_ctx.init(session_id);
            }

            // Wait for other commits.
            tokio::time::sleep(waiting_window).await;

            // Set commits.
            {
                let mut _session_ctx = self.lock().await;
                _session_ctx.set_commits().await;
            }

            // Check the number of entries to execute.
            let num_entries = {
                let _session_ctx = self.lock().await;
                _session_ctx.num_entries()
            };

            // Re-start the session if no commits found.
            if num_entries == 0 {
                continue;
            }

            // Set ctxes & lock the session.
            {
                let mut _session_ctx = self.lock().await;

                match _session_ctx.set_ctxes().await {
                    true => _session_ctx.locked(),
                    false => {
                        eprintln!("{}", "Unexpected error: Failed to set ctxes.".red());
                        continue;
                    }
                }
            }

            // Commitacks are being sent immediately after lock..

            // Spawn the opcov background task.
            {
                let session_ctx: CSESSION_CTX = Arc::clone(&self);
                tokio::spawn(async move {
                    session_ctx.opcov_task().await;
                });
            }

            // Wait for the upheld.
            if !self.await_upheld().await {
                waiting_window = ON_STAGE_WAIT_TIME_POSTUPHELDERR;
                continue;
            }

            // Post-uphold logic..

            // End of successful session.
            waiting_window = ON_STAGE_WAIT_TIME_REGULAR;
        }
    }

    async fn await_upheld(&self) -> bool {
        let start = Instant::now();

        loop {
            // Check if the upholds are ready.
            {
                let mut _session_ctx = self.lock().await;

                if _session_ctx.is_upheld_ready() {
                    _session_ctx.upheld();
                    return true;
                }
            };

            // Check for a timeout if not upheld.
            match start.elapsed() > UPHOLD_TIMEOUT {
                true => {
                    eprintln!(
                        "{}",
                        "Session timed out due to one or more missing upholds.".yellow()
                    );

                    // Appy blaming and print blame list.
                    {
                        let _session_ctx = self.lock().await;
                        _session_ctx.blame().await;
                        _session_ctx.print_blame_list();
                    }

                    return false;
                }
                false => sleep(Duration::from_millis(10)).await,
            }
        }
    }

    async fn opcov_peer_list(&self) -> Option<Vec<PEER>> {
        let key_list = {
            let dkg_manager: DKG_MANAGER = {
                let _session_ctx = self.lock().await;
                _session_ctx.dkg_manager()
            };
            let _dkg_manager = dkg_manager.lock().await;
            _dkg_manager.full_operator_list().await
        };

        let peer_list: Vec<PEER> = {
            let peer_manager: PEER_MANAGER = {
                let _session_ctx = self.lock().await;
                _session_ctx.peer_manager()
            };

            let peers: Vec<PEER> = match {
                let _peer_manager = peer_manager.lock().await;
                _peer_manager.retrieve_peers(&key_list)
            } {
                Some(peers) => peers,
                None => {
                    eprintln!(
                        "{}",
                        "Unexpected error: Failed to retrieve opcov peer list.".red()
                    );
                    return None;
                }
            };

            match peers.len() >= (key_list.len() / 2 + 1) {
                true => peers,
                false => {
                    eprintln!("{}", "Unexpected error: Insufficient opcov peers.".red());
                    return None;
                }
            }
        };

        Some(peer_list)
    }

    async fn opcov_task(&self) -> bool {
        // Return opcov.
        let opcov = {
            let _session_ctx = self.lock().await;
            match _session_ctx.opcov() {
                Some(opcov) => opcov,
                None => return false,
            }
        };

        // Return opcov peer list.
        let peer_list: Vec<PEER> = match self.opcov_peer_list().await {
            Some(list) => list,
            None => return false,
        };

        for peer in peer_list.iter() {
            let peer: PEER = Arc::clone(&peer);
            let opcov = opcov.clone();
            let session_ctx: CSESSION_CTX = Arc::clone(&self);

            // Opcov requests.
            tokio::spawn(async move {
                if let Ok(opcovack) = peer.request_opcov(opcov).await {
                    // Opcovack insertions.
                    {
                        let mut _session_ctx = session_ctx.lock().await;
                        _session_ctx.insert_opcovack(opcovack);
                    }
                }
            });
        }

        true
    }
}
