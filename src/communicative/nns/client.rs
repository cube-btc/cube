use super::relay::Relay;
use crate::transmutative::key::KeyHolder;
use nostr_sdk::prelude::*;
use std::time::Duration;

#[derive(Clone)]
pub struct NNSClient {
    nostr_client: Client,
    keys: Keys,
}

impl NNSClient {
    pub async fn new(keys: &KeyHolder) -> Self {
        let keys = keys.nostr_key_pair();
        let nostr_client = Client::new();
        {
            nostr_client.add_default_relay_list().await;
            nostr_client.connect().await;
        }
        NNSClient { nostr_client, keys }
    }

    pub async fn query_address(&self, npub: &str) -> Option<String> {
        let public_key = PublicKey::from_bech32(npub).ok()?;

        let filter = Filter::new()
            .author(public_key)
            .kind(Kind::TextNote)
            .limit(1);

        let events = {
            self.nostr_client
                .fetch_events(filter)
                .timeout(Duration::from_millis(5_000))
                .await
        };

        let last_event = match &events {
            Ok(events) => match events.iter().next() {
                Some(event) => event,
                None => return None,
            },
            Err(_) => return None,
        };

        Some(last_event.content.clone())
    }

    pub async fn publish_address(&self, ip_address: &str) -> Option<[u8; 32]> {
        let event = EventBuilder::new(Kind::TextNote, ip_address)
            .finalize(&self.keys)
            .ok()?;

        match self.nostr_client.send_event(&event).await {
            Ok(ok) => Some(ok.id().to_bytes()),
            Err(_) => None,
        }
    }
}
