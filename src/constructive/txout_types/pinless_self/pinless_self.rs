use super::default::PinlessSelfDefault;
use super::unknown::PinlessSelfUnknown;
use bitcoin::{OutPoint, TxOut};
use serde::{Deserialize, Serialize};
use serde_json::Value;

type Bytes = Vec<u8>;

/// Pinning attack-resistant self tx out type for `Swapout` entry kind.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PinlessSelf {
    // Default PinlessSelf: self account key can spend after one block.
    Default(PinlessSelfDefault),

    // Unknown PinlessSelf: scriptpubkey content is unknown, but trusted to not attempt a pinning attack.
    Unknown(PinlessSelfUnknown),
}

impl PinlessSelf {
    pub fn new_default(account_key: [u8; 32], location: Option<(OutPoint, TxOut)>) -> PinlessSelf {
        PinlessSelf::Default(PinlessSelfDefault::new(account_key, location))
    }

    pub fn new_unknown(
        custom_scriptpubkey: Vec<u8>,
        location: Option<(OutPoint, TxOut)>,
    ) -> PinlessSelf {
        PinlessSelf::Unknown(PinlessSelfUnknown::new(custom_scriptpubkey, location))
    }

    pub fn account_key(&self) -> Option<[u8; 32]> {
        match self {
            PinlessSelf::Default(pinless_self_default) => Some(pinless_self_default.account_key),
            PinlessSelf::Unknown(_) => None,
        }
    }

    pub fn location(&self) -> Option<(OutPoint, TxOut)> {
        match self {
            PinlessSelf::Default(pinless_self_default) => pinless_self_default.location(),
            PinlessSelf::Unknown(pinless_self_unknown) => pinless_self_unknown.location(),
        }
    }

    pub fn outpoint(&self) -> Option<OutPoint> {
        self.location().as_ref().map(|(outpoint, _)| *outpoint)
    }

    pub fn txout(&self) -> Option<TxOut> {
        self.location().as_ref().map(|(_, txout)| txout.clone())
    }

    pub fn calculated_scriptpubkey(&self) -> Option<Bytes> {
        match self {
            PinlessSelf::Default(pinless_self_default) => {
                pinless_self_default.calculated_scriptpubkey()
            }
            PinlessSelf::Unknown(_) => None,
        }
    }

    pub fn json(&self) -> Value {
        match self {
            PinlessSelf::Default(pinless_self_default) => pinless_self_default.json(),
            PinlessSelf::Unknown(pinless_self_unknown) => pinless_self_unknown.json(),
        }
    }

}

