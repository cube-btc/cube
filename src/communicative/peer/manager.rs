use super::peer::{Peer, PeerConnection, PeerKind, PEER, SOCKET};
use crate::{communicative::nns::client::NNSClient, inscriptive::baked, operative::Chain};
use async_trait::async_trait;
use futures::future::join_all;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

/// Guarded peer manager.
#[allow(non_camel_case_types)]
pub type PEER_MANAGER = Arc<Mutex<PeerManager>>;

pub fn coordinator_key(chain: Chain) -> [u8; 32] {
    match chain {
        Chain::Signet | Chain::Testbed => baked::SIGNET_COORDINATOR,
        Chain::Mainnet => baked::MAINNET_COORDINATOR,
    }
}

#[derive(Clone)]
pub struct PeerManager {
    chain: Chain,
    peers: HashMap<[u8; 32], PEER>,
    nns_client: NNSClient,
}

impl PeerManager {
    pub async fn new(
        chain: Chain,
        nns_client: &NNSClient,
        kind: PeerKind,
        keys: &Vec<[u8; 32]>,
    ) -> Option<PEER_MANAGER> {
        let manager_ = PeerManager {
            chain,
            peers: HashMap::<[u8; 32], PEER>::new(),
            nns_client: nns_client.to_owned(),
        };

        let mut manager = Arc::new(Mutex::new(manager_));

        manager.add_peers(kind, keys).await;

        Some(manager)
    }

    async fn insert_peer(&mut self, peer: PEER) -> bool {
        let peer_key = peer.key().await;

        if self.peers.contains_key(&peer_key) {
            return false;
        }

        self.peers.insert(peer_key, Arc::clone(&peer));

        true
    }

    pub fn chain(&self) -> Chain {
        self.chain
    }

    pub fn peers(&self) -> HashMap<[u8; 32], PEER> {
        self.peers.clone()
    }

    pub fn retrieve_peer(&self, key: [u8; 32]) -> Option<PEER> {
        Some(Arc::clone(self.peers.get(&key)?))
    }

    pub fn retrieve_peers(&self, keys: &Vec<[u8; 32]>) -> Option<Vec<PEER>> {
        let mut peers = Vec::<PEER>::new();

        for key in keys {
            if let Some(peer) = self.retrieve_peer(key.to_owned()) {
                peers.push(peer);
            }
        }
        Some(peers)
    }

    pub fn is_peer(&self, key: [u8; 32]) -> bool {
        match self.retrieve_peer(key) {
            Some(_) => return true,
            None => return false,
        }
    }

    pub async fn peer_socket(&self, key: [u8; 32]) -> Option<SOCKET> {
        let peer = self.retrieve_peer(key)?;
        let _peer = peer.lock().await;
        _peer.socket()
    }

    pub async fn is_peer_connected(&self, key: [u8; 32]) -> bool {
        let peer = match self.retrieve_peer(key) {
            Some(peer) => peer,
            None => return false,
        };

        let conn = {
            let _peer = peer.lock().await;
            _peer.connection()
        };

        match conn {
            Some(_) => return true,
            None => return false,
        }
    }
}

#[async_trait]
pub trait PeerManagerExt {
    async fn add_peers(&mut self, kind: PeerKind, keys: &Vec<[u8; 32]>) -> u64;
}

#[async_trait]
impl PeerManagerExt for PEER_MANAGER {
    /// Tries to connect to a list of peers and returns the number of peers connected.
    async fn add_peers(&mut self, kind: PeerKind, keys: &Vec<[u8; 32]>) -> u64 {
        let chain = {
            let _self = self.lock().await;
            _self.chain()
        };

        let peer_list_ = Arc::new(Mutex::new(Vec::<PEER>::new()));

        let mut tasks = vec![];

        for key in keys.iter() {
            {
                let _self = self.lock().await;
                if _self.is_peer(key.to_owned()) {
                    continue;
                }
            }

            let peer_list_ = Arc::clone(&peer_list_);
            let kind = kind.clone();
            let key = key.clone();
            let nns_client = {
                let _self = self.lock().await;
                _self.nns_client.clone()
            };

            tasks.push(tokio::spawn(async move {
                let peer: PEER = match Peer::connect(chain, kind, key, &nns_client).await {
                    Ok(peer) => peer,
                    Err(_) => return,
                };

                {
                    let mut _peer_list_ = peer_list_.lock().await;
                    _peer_list_.push(peer);
                }
            }));
        }

        join_all(tasks).await;

        let peer_list: Vec<PEER> = {
            let _peer_list = peer_list_.lock().await;
            (*_peer_list).clone()
        };

        let peer_list_len = peer_list.len() as u64;

        for peer in peer_list {
            {
                let mut _self = self.lock().await;
                _self.insert_peer(peer).await;
            }
        }

        peer_list_len
    }
}
