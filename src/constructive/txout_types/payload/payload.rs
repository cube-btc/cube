use crate::constructive::taproot::{TapLeaf, TapRoot, P2TR};
use crate::inscriptive::baked;
use crate::operative::run_args::chain::Chain;
use crate::transmutative::codec::address::encode_p2tr;
use crate::transmutative::codec::csv::CSVEncode;
use crate::transmutative::codec::csv::CSVFlag;
use crate::transmutative::codec::prefix::Prefix;
use bitcoin::hashes::Hash;
use bitcoin::{Amount, OutPoint, ScriptBuf, TxOut, Txid};
use serde::Deserialize;
use serde::Serialize;

// A type alias for bytes.
type Bytes = Vec<u8>;

type TapLeafHash = [u8; 32];
type TapScript = Vec<u8>;
type ControlBlock = Vec<u8>;

#[derive(Clone, Serialize, Deserialize)]
pub struct Payload {
    pub engine_key: [u8; 32],
    pub payload_bytes: Vec<u8>,
    pub location: Option<(OutPoint, TxOut)>,
}

impl Payload {
    /// Creates a new Payload struct.
    pub fn new(
        engine_key: [u8; 32],
        payload_bytes: Vec<u8>,
        location: Option<(OutPoint, TxOut)>,
    ) -> Payload {
        Payload {
            engine_key,
            payload_bytes,
            location,
        }
    }

    /// Serializes the Payload with bincode.
    pub fn serialize(&self) -> Option<Vec<u8>> {
        bincode::serde::encode_to_vec(self, bincode::config::standard()).ok()
    }

    /// Deserializes a Payload from bincode bytes.
    pub fn deserialize(bytes: &[u8]) -> Option<Payload> {
        bincode::serde::decode_from_slice::<Payload, _>(bytes, bincode::config::standard())
            .ok()
            .map(|(payload, _)| payload)
    }

    /// Returns the location of the Payload.
    pub fn location(&self) -> Option<(OutPoint, TxOut)> {
        self.location.clone()
    }

    /// Returns the outpoint for the Payload.
    pub fn outpoint(&self) -> Option<OutPoint> {
        self.location
            .as_ref()
            .map(|(outpoint, _txout)| outpoint.clone())
    }

    /// Returns the txout for the Payload.
    pub fn txout(&self) -> Option<TxOut> {
        self.location
            .as_ref()
            .map(|(_outpoint, txout)| txout.clone())
    }

    /// Returns the scriptpubkey for the Payload.
    pub fn calculated_scriptpubkey(&self) -> Option<Bytes> {
        return_payload_scriptpubkey(self.engine_key, &self.payload_bytes)
    }

    /// Returns the P2TR script-path spend elements for the Payload.
    pub fn p2tr_script_path_spend_elements(&self) -> (TapLeafHash, TapScript, ControlBlock) {
        // For Payload we construct a single-leaf, script-path-only TapRoot.
        let taproot = self
            .taproot()
            .expect("This should never happen: Payload must always have a TapRoot for P2TR script-path spends");

        let tree = taproot
            .tree()
            .expect("This should never happen: TapRoot for Payload must contain a TapTree");

        let leaves = tree.leaves();
        let tapleaf = leaves.first().expect(
            "This should never happen: TapTree for Payload must contain at least one TapLeaf",
        );

        let tapleaf_hash: TapLeafHash = tapleaf.tapleaf_hash();
        let tapscript: TapScript = tapleaf.tap_script();

        let control_block_bytes: ControlBlock = taproot
            .control_block(0)
            .expect("This should never happen: TapRoot for Payload must produce a control block for leaf index 0")
            .to_vec();

        (tapleaf_hash, tapscript, control_block_bytes)
    }
}

/// Returns a tapscript for the Payload.
pub fn return_payload_tapscript(engine_key: [u8; 32], payload_bytes: &Vec<u8>) -> Option<Vec<u8>> {
    // 1 Initialize the tapscript.
    let mut tapscript = Vec::<u8>::new();

    // 2 Encode the tapscript
    {
        // 2.1 Encode the payload bytes
        {
            // 2.1.1 OP_0 (0x00)
            tapscript.push(0x00);

            // 2.1.2 OP_IF (0x63)
            tapscript.push(0x63);

            // 2.1.3 Push the payload bytes
            tapscript.extend(payload_bytes.prefix_pushdata());

            // 2.1.4 OP_ENDIF (0x68)
            tapscript.push(0x68);
        }

        // 2.2 OP_IF (0x63)
        tapscript.push(0x63);

        // 2.3 Engine spend clause: <Engine Key> OP_CHECKSIG
        {
            // 2.3.1 Push engine key
            tapscript.push(0x20); // OP_PUSHDATA_32
            tapscript.extend(engine_key); // Engine Key 32-bytes

            // 2.3.2 OP_CHECKSIG (0xac)
            tapscript.push(0xac);
        }

        // 2.4 OP_ELSE (0x67)
        tapscript.push(0x67);

        // 2.5 Anyone-can-spend after timeout clause: <3 months> OP_CHECKSEQUENCEVERIFY OP_DROP OP_1
        // This forces Engine to not halt operations.
        {
            // 2.5.1 <3 months>
            tapscript.extend(Vec::<u8>::csv_num_encode(CSVFlag::CSVThreeMonths));

            // 2.5.2 OP_CHECKSEQUENCEVERIFY (0xb2)
            tapscript.push(0xb2);

            // 2.5.3 OP_DROP (0x75)
            tapscript.push(0x75);

            // 2.5.4 OP_1 (0x51)
            tapscript.push(0x51);
        }

        // 2.6 OP_ENDIF (0x68)
        tapscript.push(0x68);
    }

    // 3 Return the tapscript
    Some(tapscript)
}

/// Returns a taproot for the Payload.
pub fn return_payload_taproot(engine_key: [u8; 32], payload_bytes: &Vec<u8>) -> Option<TapRoot> {
    // 1 Get the tapscript.
    let tapscript = return_payload_tapscript(engine_key, payload_bytes)?;

    // 2 Construct the tapleaf.
    let tapleaf = TapLeaf::new(tapscript);

    // 3 Construct the taproot.
    let taproot = TapRoot::script_path_only_single(tapleaf);

    // 4 Return the taproot.
    Some(taproot)
}

/// Returns a scriptpubkey for the Payload.
pub fn return_payload_scriptpubkey(engine_key: [u8; 32], payload_bytes: &Vec<u8>) -> Option<Bytes> {
    // 1 Get the taproot.
    let taproot = return_payload_taproot(engine_key, payload_bytes)?;

    // 2 Return the scriptpubkey
    taproot.spk()
}

impl P2TR for Payload {
    fn taproot(&self) -> Option<TapRoot> {
        return_payload_taproot(self.engine_key, &self.payload_bytes)
    }
}

/// Returns a genesis engine payload address.
pub fn genesis_payload_address(chain: Chain) -> String {
    // 1 Get the engine key and payload bytes.
    let engine_key: [u8; 32] = match chain {
        Chain::Testbed => baked::SIGNET_ENGINE_PUBLIC_KEY,
        Chain::Signet => baked::SIGNET_ENGINE_PUBLIC_KEY,
        Chain::Mainnet => baked::MAINNET_ENGINE_PUBLIC_KEY,
    };

    // 2 Get the payload bytes.
    let payload_bytes = baked::GENESIS_INSCRIPTION.to_vec();

    // 3 Construct the genesis payload without location.
    let genesis_payload_without_location = Payload::new(engine_key, payload_bytes, None);

    // 4 Get the scriptpubkey for the genesis payload.
    let genesis_payload_taproot = genesis_payload_without_location
        .taproot()
        .expect("Failed to get taproot.");

    // 5 Get the tweaked key for the genesis payload.
    let genesis_payload_taproot_key: [u8; 32] = genesis_payload_taproot
        .tweaked_key()
        .expect("Failed to get tweaked key.")
        .serialize_xonly();

    // 6 Encode the tweaked key into an address.
    let genesis_payload_address =
        encode_p2tr(chain, genesis_payload_taproot_key).expect("Failed to encode p2tr address.");

    // 7 Return the genesis payload address.
    genesis_payload_address
}

/// Returns a genesis engine payload.
pub fn genesis_payload(chain: Chain) -> Payload {
    // 1 Get the engine key and payload bytes.
    let engine_key: [u8; 32] = match chain {
        Chain::Testbed => baked::SIGNET_ENGINE_PUBLIC_KEY,
        Chain::Signet => baked::SIGNET_ENGINE_PUBLIC_KEY,
        Chain::Mainnet => baked::MAINNET_ENGINE_PUBLIC_KEY,
    };

    // 2 Get the payload bytes.
    let payload_bytes = baked::GENESIS_INSCRIPTION.to_vec();

    // 3 Get the genesis payload txid and vout.
    let genesis_payload_txid: [u8; 32] = match chain {
        Chain::Testbed => baked::SIGNET_GENESIS_PAYLOAD_TX_ID,
        Chain::Signet => baked::SIGNET_GENESIS_PAYLOAD_TX_ID,
        Chain::Mainnet => baked::MAINNET_GENESIS_PAYLOAD_TX_ID,
    };

    // 4 Get the genesis payload vout.
    let genesis_payload_vout: u32 = match chain {
        Chain::Testbed => baked::SIGNET_GENESIS_PAYLOAD_VOUT,
        Chain::Signet => baked::SIGNET_GENESIS_PAYLOAD_VOUT,
        Chain::Mainnet => baked::MAINNET_GENESIS_PAYLOAD_VOUT,
    };

    // 5 Construct the genesis payload without location.
    let genesis_payload_without_location = Payload::new(engine_key, payload_bytes, None);

    // 6 Get the scriptpubkey for the genesis payload.
    let genesis_payload_scriptpubkey = genesis_payload_without_location
        .calculated_scriptpubkey()
        .expect("Failed to get scriptpubkey.");

    // 7 Construct the location for the genesis payload.
    let location = (
        OutPoint::new(
            Txid::from_raw_hash(Hash::from_byte_array(genesis_payload_txid)),
            genesis_payload_vout,
        ),
        TxOut {
            value: Amount::from_sat(0),
            script_pubkey: ScriptBuf::from(genesis_payload_scriptpubkey),
        },
    );

    // 8 Construct the genesis payload with location.
    let genesis_payload = Payload::new(
        engine_key,
        genesis_payload_without_location.payload_bytes.clone(),
        Some(location),
    );

    // 9 Return the genesis payload.
    genesis_payload
}
