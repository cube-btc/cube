use bitcoin::hashes::Hash;
use bitcoin::{Amount, OutPoint, ScriptBuf, TxOut, Txid};

pub trait OutpointExt {
    /// Returns the OutPoint as a 36 byte array.
    fn bytes_36(&self) -> [u8; 36];
    /// Returns the OutPoint from a 36 byte array.
    fn from_bytes36(bytes: &[u8; 36]) -> Option<OutPoint>;

    /// Returns the OutPoint's transaction hash.
    fn txhash(&self) -> [u8; 32];
    /// Returns the OutPoint's output index.
    fn vout(&self) -> u32;
}

impl OutpointExt for OutPoint {
    fn bytes_36(&self) -> [u8; 36] {
        let mut bytes = [0; 36];
        bytes[..32].copy_from_slice(&self.txid.to_byte_array());
        bytes[32..].copy_from_slice(&self.vout.to_le_bytes());
        bytes
    }

    fn from_bytes36(bytes: &[u8; 36]) -> Option<OutPoint> {
        let txid_bytes = bytes[..32].try_into().ok()?;
        let vout_bytes = bytes[32..].try_into().ok()?;
        let txid = Txid::from_byte_array(txid_bytes);
        let vout = u32::from_le_bytes(vout_bytes);
        Some(OutPoint::new(txid, vout))
    }

    fn txhash(&self) -> [u8; 32] {
        self.txid.to_byte_array()
    }

    fn vout(&self) -> u32 {
        self.vout
    }

}

pub trait TxOutExt {
    /// Returns the TxOut as a byte array.
    fn bytes(&self) -> Vec<u8>;
    /// Returns the TxOut from a byte array.
    fn from_bytes(bytes: &[u8]) -> Option<TxOut>;

    /// Returns the TxOut's scriptpubkey.
    fn scriptpubkey(&self) -> Vec<u8>;

    /// Returns the TxOut's value.
    fn value_in_satoshis(&self) -> u64;
    
}

impl TxOutExt for TxOut {
    fn bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Value.
        let value: [u8; 8] = self.value.to_sat().to_le_bytes();

        // Script pubkey.
        let script_pubkey = self.script_pubkey.as_bytes();

        // Script pubkey length.
        let script_pubkey_len: [u8; 1] = (script_pubkey.len() as u8).to_le_bytes();

        // Extend bytes.
        bytes.extend_from_slice(&value);
        bytes.extend_from_slice(&script_pubkey_len);
        bytes.extend_from_slice(&script_pubkey);

        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Option<TxOut> {
        let value_bytes = bytes[..8].try_into().ok()?;
        let value = u64::from_le_bytes(value_bytes);

        let script_pubkey_len_bytes = bytes[8..9].try_into().ok()?;
        let script_pubkey_len = u8::from_le_bytes(script_pubkey_len_bytes);
        let script_pubkey = bytes[9..9 + script_pubkey_len as usize].to_vec();

        let txout = TxOut {
            value: Amount::from_sat(value),
            script_pubkey: ScriptBuf::from(script_pubkey),
        };

        Some(txout)
    }

    fn scriptpubkey(&self) -> Vec<u8> {
        self.script_pubkey.as_bytes().to_vec()
    }

    fn value_in_satoshis(&self) -> u64 {
        self.value.to_sat()
    }
}
