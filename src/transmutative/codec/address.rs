use crate::operative::Chain;
use bech32::{segwit, Hrp};

type ScriptPubKey = Vec<u8>;

/// Returns the Human-Readable Part (HRP) for a given network.
///
/// This function takes a network type and returns the corresponding HRP.
///
/// # Arguments
///

fn hrp_from_chain(chain: Chain) -> Option<Hrp> {
    match chain {
        Chain::Signet | Chain::Testbed => Hrp::parse("tb").ok(),
        Chain::Mainnet => Hrp::parse("bc").ok(),
    }
}

/// Encodes a Taproot key into a Bech32-encoded Bitcoin address.
///
/// This function takes a network type and a Taproot key, and returns the corresponding
/// Bech32-encoded Bitcoin address if the key is valid.
///
/// # Arguments
///
pub fn encode_p2tr(chain: Chain, taproot_key: [u8; 32]) -> Option<String> {
    let hrp = hrp_from_chain(chain)?;
    let address = segwit::encode(hrp, segwit::VERSION_1, &taproot_key).ok()?;

    Some(address)
}

/// Encodes a P2WSH witness program into a Bech32-encoded Bitcoin address.
///
/// This function takes a network type and a P2WSH witness program, and returns the corresponding
/// Bech32-encoded Bitcoin address if the witness program is valid.
///
/// # Arguments
///
pub fn encode_p2wsh(chain: Chain, witness_program: [u8; 32]) -> Option<String> {
    let hrp = hrp_from_chain(chain)?;
    let address = segwit::encode(hrp, segwit::VERSION_0, &witness_program).ok()?;

    Some(address)
}

/// Encodes a P2WPKH witness program into a Bech32-encoded Bitcoin address.
///
/// This function takes a network type and a P2WPKH witness program, and returns the corresponding
/// Bech32-encoded Bitcoin address if the witness program is valid.
///
/// # Arguments
///
pub fn encode_p2wpkh(chain: Chain, witness_program: [u8; 20]) -> Option<String> {
    let hrp = hrp_from_chain(chain)?;
    let address = segwit::encode(hrp, segwit::VERSION_0, &witness_program).ok()?;

    Some(address)
}

/// Decodes a Bech32-encoded Bitcoin address into a ScriptPubKey.
///
/// This function takes a Bitcoin address and a network type, and returns the corresponding
/// ScriptPubKey if the address is valid. Legacy addresses are not supported.
///
/// # Arguments
///
/// * `network` - The network type of the address.
/// * `address` - The Bech32-encoded Bitcoin address to decode.
///
/// # Returns
///

pub fn address_to_spk(chain: Chain, address: &str) -> Option<ScriptPubKey> {
    let mut spk = Vec::<u8>::new();

    let (hrp, version, program) = match segwit::decode(&address) {
        Ok(result) => result,
        Err(_) => return None,
    };

    // Check if the network is valid
    match chain {
        Chain::Signet | Chain::Testbed => {
            if hrp != Hrp::parse("tb").expect("invalid hrp") {
                return None;
            }
        }
        Chain::Mainnet => {
            if hrp != Hrp::parse("bc").expect("invalid hrp") {
                return None;
            }
        }
    }

    // Match the version
    match version {
        segwit::VERSION_0 => {
            // Segwit version 0
            spk.push(0x00);

            match program.len() {
                20 => {
                    // P2WPKH
                    spk.push(0x14);
                    spk.extend(&program);
                }
                32 => {
                    // P2WSH
                    spk.push(0x20);
                    spk.extend(&program);
                }
                _ => {
                    return None;
                }
            }
        }
        segwit::VERSION_1 => {
            // Segwit version 1
            spk.push(0x51);

            match program.len() {
                32 => {
                    // P2TR
                    spk.push(0x20);
                    spk.extend(&program);
                }
                _ => {
                    return None;
                }
            }
        }
        _ => {
            return None;
        }
    }

    Some(spk)
}
