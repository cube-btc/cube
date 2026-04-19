use crate::{constructive::entry::entry_kinds::call::call::Call, transmutative::hash::Hash};
use crate::constructive::entry::entry_kinds::liftup::liftup::Liftup;
use crate::transmutative::hash::HashTag;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents an `Entry`.
///
/// An `Entry` is a container for specific actions, such as calling a `Contract` or moving coins.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Entry {
    //Move(MoveEntry),
    Call(Call),
    //Add(AddEntry),
    //Sub(SubEntry),
    Liftup(Liftup),
    //Swapout(SwapoutEntry),
    //Deploy(DeployEntry),
    //Config(ConfigEntry),
    //Nop(NopEntry),
    //Fail(FailEntry),
}

impl Entry {
    /// Creates a new call entry.
    pub fn new_call(call: Call) -> Self {
        Self::Call(call)
    }

    /// Creates a new liftup entry.
    pub fn new_liftup(liftup: Liftup) -> Self {
        Self::Liftup(liftup)
    }

    /// Returns this entry as a JSON object.
    pub fn json(&self) -> Value {
        match self {
            Entry::Call(call) => call.json(),
            Entry::Liftup(liftup) => liftup.json(),
        }
    }

    /// Returns the entry ID.
    pub fn entry_id(&self, batch_height: u64) -> Option<[u8; 32]> {
        match self {
            Entry::Call(_) => panic!("Not implemented yet."),
            Entry::Liftup(liftup) => {
                // 1 Initialize the preimage.
                let mut preimage = Vec::<u8>::new();

                // 2 Push batch height to the preimage.
                preimage.extend(batch_height.to_le_bytes());

                // 3 Push liftup sighash to the preimage.
                preimage.extend(liftup.sighash().ok()?);

                // 4 Hash the preimage.
                let hash = preimage.hash(Some(HashTag::LiftupEntryID));

                // 5 Return the hash.
                Some(hash)
            },
        }
    }

    /// Serializes this entry with bincode (same config as wire payloads elsewhere).
    pub fn serialize(&self) -> Option<Vec<u8>> {
        bincode::serde::encode_to_vec(self, bincode::config::standard()).ok()
    }

    /// Deserializes an entry from bincode bytes.
    pub fn deserialize(bytes: &[u8]) -> Option<Self> {
        bincode::serde::decode_from_slice::<Self, _>(bytes, bincode::config::standard())
            .ok()
            .map(|(entry, _)| entry)
    }
}
