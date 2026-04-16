type Bytes = Vec<u8>;

pub fn encode_varint(value: u64) -> Bytes {
    match value {
        0..=252 => vec![value as u8],
        253..=65535 => {
            let mut encoded = vec![0xfd];
            encoded.extend((value as u16).to_le_bytes());
            encoded
        }
        65536..=4294967295 => {
            let mut encoded = vec![0xfe];
            encoded.extend((value as u32).to_le_bytes());
            encoded
        }
        _ => {
            let mut encoded = vec![0xff];
            encoded.extend(value.to_le_bytes());
            encoded
        }
    }
}
