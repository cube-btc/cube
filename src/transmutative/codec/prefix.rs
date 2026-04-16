type Bytes = Vec<u8>;
use super::varint::encode_varint;

pub trait Prefix {
    // Interpret bytes as stack push and prefix them with OP_PUSHDATA.
    // https://en.bitcoin.it/wiki/Script
    fn prefix_pushdata(&self) -> Bytes;

    // Interpret bytes as common protocol serialization and prefix them with a variable-length integer.
    // https://en.bitcoin.it/wiki/Protocol_documentation#Variable_length_integer
    fn prefix_compact_size(&self) -> Bytes;
}

impl Prefix for Bytes {
    fn prefix_pushdata(&self) -> Bytes {
        let mut bytes = Vec::<u8>::new();
        let data_len = self.len();

        if data_len == 1 && (&self[0] == &0x81 || &self[0] <= &16) {
            // Check minimal push.
            // https://github.com/bitcoin/bitcoin/blob/master/src/script/script.cpp#L366
            match &self[0] {
                // OP_1NEGATE
                0x81 => bytes.push(0x4f),
                // OP_0
                0x00 => bytes.push(0x00),
                // OP_1
                0x01 => bytes.push(0x51),
                // OP_2
                0x02 => bytes.push(0x52),
                // OP_3
                0x03 => bytes.push(0x53),
                // OP_4
                0x04 => bytes.push(0x54),
                // OP_5
                0x05 => bytes.push(0x55),
                // OP_6
                0x06 => bytes.push(0x56),
                // OP_7
                0x07 => bytes.push(0x57),
                // OP_8
                0x08 => bytes.push(0x58),
                // OP_9
                0x09 => bytes.push(0x59),
                // OP_10
                0x0a => bytes.push(0x5a),
                // OP_11
                0x0b => bytes.push(0x5b),
                // OP_12
                0x0c => bytes.push(0x5c),
                // OP_13
                0x0d => bytes.push(0x5d),
                // OP_14
                0x0e => bytes.push(0x5e),
                // OP_15
                0x0f => bytes.push(0x5f),
                // OP_16
                0x10 => bytes.push(0x60),
                _ => (),
            }
        } else {
            match data_len {
                0..=75 => bytes.extend(vec![data_len as u8]),
                76..=255 => {
                    bytes.extend([0x4c]);
                    bytes.extend([data_len as u8]);
                }
                256..=65535 => {
                    bytes.extend(vec![0x4d]);

                    let x_bytes: [u8; 2] = (data_len as u16).to_le_bytes();
                    bytes.extend(x_bytes);
                }
                65536..=4294967295 => {
                    bytes.extend([0x4e]);

                    let x_bytes: [u8; 4] = (data_len as u32).to_le_bytes();
                    bytes.extend(x_bytes);
                }
                _ => panic!("The data cannot be prefixed because it is too large."),
            }
            bytes.extend(self);
        }
        bytes
    }

    fn prefix_compact_size(&self) -> Bytes {
        let mut bytes = encode_varint(self.len() as u64);
        bytes.extend(self);
        bytes
    }
}