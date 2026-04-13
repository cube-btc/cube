use crate::constructive::taproot::{TapLeaf, TapRoot, P2TR};
use crate::transmutative::codec::csv::CSVEncode;
use crate::transmutative::codec::csv::CSVFlag;
use crate::transmutative::codec::prefix::Prefix;
use serde::Deserialize;
use serde::Serialize;

// A type alias for bytes.
type Bytes = Vec<u8>;

#[derive(Clone, Serialize, Deserialize)]
pub struct Payload {
    pub engine_key: [u8; 32],
    pub payload_bytes: Vec<u8>,
}

impl Payload {
    /// Creates a new Payload struct.
    pub fn new(engine_key: [u8; 32], payload_bytes: Vec<u8>) -> Payload {
        Payload {
            engine_key,
            payload_bytes,
        }
    }

    /// Returns the scriptpubkey for the Payload struct.
    pub fn scriptpubkey(&self) -> Option<Bytes> {
        return_payload_scriptpubkey(self.engine_key, &self.payload_bytes)
    }
}

/// Returns a taproot for the Payload struct.
pub fn return_payload_taproot(engine_key: [u8; 32], payload_bytes: &Vec<u8>) -> Option<TapRoot> {
    // 1 Construct the tapscript
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
            tapscript.extend(Vec::<u8>::csv_script(CSVFlag::CSVThreeMonths));

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

    // 3 Construct the tapleaf
    let tapleaf = TapLeaf::new(tapscript);

    // 4 Construct the taproot
    let taproot = TapRoot::script_path_only_single(tapleaf);

    // 5 Return the taproot
    Some(taproot)
}

/// Returns a scriptpubkey for the Payload struct.
pub fn return_payload_scriptpubkey(engine_key: [u8; 32], payload_bytes: &Vec<u8>) -> Option<Bytes> {
    // 1 Construct the taproot
    let taproot = return_payload_taproot(engine_key, payload_bytes)?;

    // 2 Return the scriptpubkey
    taproot.spk()
}

impl P2TR for Payload {
    fn taproot(&self) -> Option<TapRoot> {
        return_payload_taproot(self.engine_key, &self.payload_bytes)
    }
}
