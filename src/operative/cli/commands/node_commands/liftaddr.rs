use crate::constructive::txo::lift::lift_versions::liftv1::{
    return_liftv1_scriptpubkey, return_liftv1_taproot,
};
use crate::constructive::txo::lift::lift_versions::liftv2::{
    return_liftv2_scriptpubkey, return_liftv2_taproot,
};
use crate::operative::run_args::chain::Chain;
use crate::transmutative::codec::address::encode_p2tr;
use colored::Colorize;
use hex;
use serde_json::{json, to_string_pretty};

// liftaddr
pub fn liftaddr_command(chain: Chain, engine_key: [u8; 32], self_account_key: [u8; 32]) {
    // 1 Get the liftv1 scriptpubkey.
    let liftv1_scriptpubkey: Vec<u8> =
        match return_liftv1_scriptpubkey(self_account_key, engine_key) {
            Some(spk) => spk,
            None => {
                println!("{}", "Error getting liftv1 scriptpubkey.".red());
                return;
            }
        };

    // 2 Get the liftv1 tweaked taproot key.
    let liftv1_tweaked_taproot_key: [u8; 32] =
        match return_liftv1_taproot(self_account_key, engine_key) {
            Some(taproot) => match taproot.tweaked_key() {
                Some(tweaked_key) => tweaked_key.serialize_xonly(),
                None => {
                    println!("{}", "Error getting liftv1 tweaked key.".red());
                    return;
                }
            },
            None => {
                println!("{}", "Error getting liftv1 scriptpubkey.".red());
                return;
            }
        };

    // 3 Encode the liftv1 scriptpubkey into an address.
    let liftv1_addr = match encode_p2tr(chain, liftv1_tweaked_taproot_key) {
        Some(addr) => addr,
        None => {
            println!("{}", "Error encoding liftv1 address.".red());
            return;
        }
    };

    // 4 Get the liftv2 scriptpubkey.
    let liftv2_scriptpubkey: Vec<u8> =
        match return_liftv2_scriptpubkey(self_account_key, engine_key) {
            Some(spk) => spk,
            None => {
                println!("{}", "Error getting liftv2 scriptpubkey.".red());
                return;
            }
        };

    // 5 Get the liftv2 tweaked taproot key.
    let liftv2_tweaked_taproot_key: [u8; 32] =
        match return_liftv2_taproot(self_account_key, engine_key) {
            Some(taproot) => match taproot.tweaked_key() {
                Some(tweaked_key) => tweaked_key.serialize_xonly(),
                None => {
                    println!("{}", "Error getting liftv2 tweaked key.".red());
                    return;
                }
            },
            None => {
                println!("{}", "Error getting liftv2 scriptpubkey.".red());
                return;
            }
        };

    // 6 Encode the liftv2 tweaked taproot key into an address.
    let liftv2_addr = match encode_p2tr(chain, liftv2_tweaked_taproot_key) {
        Some(addr) => addr,
        None => {
            println!("{}", "Error encoding liftv2 address.".red());
            return;
        }
    };

    // 7 Construct the JSON value.
    let json_value = json!({
        "liftv1": {
            "address": liftv1_addr,
            "scriptpubkey": hex::encode(&liftv1_scriptpubkey),
        },
        "liftv2": {
            "address": liftv2_addr,
            "scriptpubkey": hex::encode(&liftv2_scriptpubkey),
        },
    });

    // 8 Print the JSON value.
    println!(
        "{}",
        to_string_pretty(&json_value).expect("serde_json::Value should serialize")
    );
}
