use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize)]
pub struct Projector {
    pub scriptpubkey: Vec<u8>,
    pub satoshi_amount: u64,
}
