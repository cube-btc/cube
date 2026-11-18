use nostr_sdk::prelude::*;

pub const DEFAULT_RELAY_LIST: [&str; 5] = [
    "wss://relay.damus.io",
    "wss://relay.primal.net",
    "wss://nostr.256k1.dev",
    "wss://nostr.fmt.wiz.biz",
    "wss://relay.nostr.ch",
];

pub trait Relay {
    fn add_default_relay_list(&self) -> impl std::future::Future<Output = ()> + Send;
}

impl Relay for Client {
    fn add_default_relay_list(&self) -> impl std::future::Future<Output = ()> + Send {
        async move {
            // Add the list of default relays.
            for relay in DEFAULT_RELAY_LIST {
                let _ = self.add_relay(relay).await;
            }
        }
    }
}
