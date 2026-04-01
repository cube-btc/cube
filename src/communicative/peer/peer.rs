use crate::{
    communicative::{
        nns::client::NNSClient,
        tcp::{
            client::TCPClient,
            tcp::{connect_nns, TCPError},
        },
    },
    operative::Chain,
};
use async_trait::async_trait;
use colored::Colorize;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::Mutex;

/// Guarded TCP socket.
#[allow(non_camel_case_types)]
pub type SOCKET = Arc<Mutex<tokio::net::TcpStream>>;

#[derive(Copy, Clone, PartialEq)]
pub enum PeerKind {
    Node,
    Engine,
}

impl PeerKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            PeerKind::Node => "Node",
            PeerKind::Engine => "Engine",
        }
    }
}

#[derive(Clone)]
pub struct Peer {
    chain: Chain,
    kind: PeerKind,
    key: [u8; 32],
    nns_client: NNSClient,
    connection: Option<(SOCKET, SocketAddr)>,
}

/// Guarded peer.
#[allow(non_camel_case_types)]
pub type PEER = Arc<Mutex<Peer>>;

impl Peer {
    pub async fn connect(
        chain: Chain,
        kind: PeerKind,
        key: [u8; 32],
        nns_client: &NNSClient,
    ) -> Result<PEER, TCPError> {
        let (socket_, addr) = {
            match connect_nns(key, &nns_client, chain).await {
                Ok(socket) => {
                    let addr = match socket.peer_addr() {
                        Ok(addr) => addr,
                        Err(_) => return Err(TCPError::ConnErr),
                    };

                    (socket, addr)
                }
                Err(_) => return Err(TCPError::ConnErr),
            }
        };

        let socket: SOCKET = Arc::new(Mutex::new(socket_));

        let connection = Some((socket, addr));

        let peer_ = Peer {
            chain,
            kind,
            key,
            connection,
            nns_client: nns_client.clone(),
        };

        let peer = Arc::new(Mutex::new(peer_));

        peer.set_uptimer().await;

        Ok(peer)
    }

    pub fn chain(&self) -> Chain {
        self.chain
    }

    pub fn kind(&self) -> PeerKind {
        self.kind
    }

    pub fn key(&self) -> [u8; 32] {
        self.key
    }

    pub fn nns_client(&self) -> NNSClient {
        self.nns_client.clone()
    }

    pub fn connection(&self) -> Option<(SOCKET, SocketAddr)> {
        self.connection.clone()
    }

    pub fn connected(&self) -> bool {
        match self.connection() {
            Some(_) => true,
            None => false,
        }
    }

    pub fn socket(&self) -> Option<SOCKET> {
        Some(Arc::clone(&self.connection()?.0))
    }

    pub fn set_connection(&mut self, connection: Option<(SOCKET, SocketAddr)>) {
        self.connection = connection;
    }

    pub fn addr(&self) -> String {
        match self.connection() {
            Some(connection) => {
                return format!("{}:{}", connection.1.ip(), connection.1.port());
            }
            None => {
                return "Dead.".to_string();
            }
        };
    }
}

#[async_trait]
pub trait PeerConnection {
    async fn key(&self) -> [u8; 32];
    async fn socket(&self) -> Option<SOCKET>;
    async fn disconnection(&self);
    async fn reconnect(&self);
    async fn set_uptimer(&self);
}

#[async_trait]
impl PeerConnection for PEER {
    async fn key(&self) -> [u8; 32] {
        let _self = self.lock().await;
        _self.key()
    }

    async fn socket(&self) -> Option<SOCKET> {
        let _self = self.lock().await;
        _self.socket()
    }

    async fn disconnection(&self) {
        loop {
            loop {
                match self.ping().await {
                    Ok(_) => break,
                    Err(_) => {
                        let mut failure_iter: u8 = 0;
                        loop {
                            if failure_iter < 3 {
                                failure_iter += 1;
                                tokio::time::sleep(Duration::from_secs(5)).await;
                                continue;
                            } else {
                                let mut _peer = self.lock().await;
                                _peer.set_connection(None);

                                return ();
                            }
                        }
                    }
                }
            }
            tokio::time::sleep(Duration::from_secs(15)).await;
        }
    }

    async fn reconnect(&self) {
        let chain = {
            let _self = self.lock().await;
            _self.chain()
        };

        let (socket_, addr) = {
            loop {
                let (nns_key, nns_client) = {
                    let _peer = self.lock().await;
                    (_peer.key(), _peer.nns_client())
                };

                match connect_nns(nns_key, &nns_client, chain).await {
                    Ok(socket) => {
                        let addr = match socket.peer_addr() {
                            Ok(addr) => addr,
                            Err(_) => {
                                tokio::time::sleep(Duration::from_secs(5)).await;
                                continue;
                            }
                        };

                        break (socket, addr);
                    }
                    Err(_) => {
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                }
            }
        };

        let socket: SOCKET = Arc::new(Mutex::new(socket_));

        {
            let mut _peer = self.lock().await;
            _peer.set_connection(Some((socket, addr)));
        }
    }

    async fn set_uptimer(&self) {
        let peer = Arc::clone(&self);

        tokio::spawn(async move {
            loop {
                // Wait until disconnection.
                let (peer_kind_str, peer_addr) = {
                    let _peer = peer.lock().await;
                    (_peer.kind().as_str(), _peer.addr())
                };
                let _ = peer.disconnection().await;
                println!(
                    "{}",
                    format!(
                        "{} '{}' disconnected. Trying to connect again..",
                        peer_kind_str, peer_addr
                    )
                    .yellow()
                );

                // Re-connect upon disconnection
                let _ = peer.reconnect().await;
                let (peer_kind_str, peer_addr) = {
                    let _peer = peer.lock().await;
                    (_peer.kind().as_str(), _peer.addr())
                };
                println!(
                    "{}",
                    format!("{} '{}' re-connected.", peer_kind_str, peer_addr).green()
                );
            }
        });
    }
}
