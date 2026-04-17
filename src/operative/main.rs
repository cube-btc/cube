use colored::Colorize;
use cube::constructive::txout_types::payload::payload::Payload;
use cube::constructive::taproot::P2TR;
use cube::inscriptive::baked;
use cube::transmutative::codec::address::encode_p2tr;
use cube::{
    communicative::rpc::bitcoin_rpc::bitcoin_rpc_holder::BitcoinRPCHolder,
    operative::{
        run_args::{
            chain::Chain, operating_kind::OperatingKind, resource_mode::ResourceMode,
            sync_mode::SyncMode,
        },
        runner::runner,
    },
    transmutative::{
        key::{FromNostrKeyStr, KeyHolder, ToNostrKeyStr},
        secp::schnorr::generate_secret,
    },
};
use serde_json::json;
use std::{env, io::BufRead};

fn main() {
    // 1 Parse arguments.
    let args: Vec<String> = env::args().collect();

    // 2 Match the arguments length.
    match args.len() {
        // 2.a Generate a random secret key and print it as an nsec.
        2 => gensec(&args),

        // 2.b Print genesis parameters.
        3 => genesis(&args),

        // 2.c Run the appropriate mode based on the arguments.
        8 => run(&args),

        // 2.d Invalid arguments.
        _ => print_correct_usage(),
    }
}

/// Prints the nsec in a decorative frame with WoT branding.
fn print_nsec_frame(nsec: &str) {
    let width = 70;
    let corner_tl = "╔";
    let corner_tr = "╗";
    let corner_bl = "╚";
    let corner_br = "╝";
    let horizontal = "═";
    let vertical = "║";
    let ornament = "✦";
    let flower = "✿";

    // Top border with WoT
    // Format: "{}{} {} {} {} {}{}" = corner + padding + " " + ornament + " " + text + " " + ornament + " " + padding + corner
    // Spaces: 1 + 1 + 1 + 1 + 1 = 5 spaces total
    // Total: 1 + padding + 1 + 1 + 1 + text_len + 1 + 1 + 1 + right_padding + 1 = padding + right_padding + text_len + 9
    let wot = "WoT";
    let wot_len = wot.len();
    let fixed = 2 + 5 + 2 + wot_len; // 2 corners + 5 spaces + 2 ornaments + text = 9 + text_len
    let total_padding = width - fixed;
    let padding = total_padding / 2;
    let right_padding = total_padding - padding;
    let top_line = format!(
        "{}{} {} {} {} {}{}",
        corner_tl,
        horizontal.repeat(padding),
        ornament,
        wot,
        ornament,
        horizontal.repeat(right_padding),
        corner_tr
    );
    println!("{}", top_line.magenta());

    // Empty line with flowers
    println!(
        "{}{}{}{}{}",
        vertical.magenta(),
        flower.magenta(),
        " ".repeat(width - 5),
        flower.magenta(),
        vertical.magenta()
    );

    // Nsec line - centered with flowers
    let inner_width = width - 5; // Account for 2 verticals + 2 flowers + 1 extra space
    let nsec_len = nsec.chars().count();
    let nsec_padding = (inner_width - nsec_len) / 2;
    let nsec_right = inner_width - nsec_padding - nsec_len;
    let nsec_line = format!(
        "{}{}{}{}{}{}{}",
        vertical.magenta(),
        flower.magenta(),
        " ".repeat(nsec_padding),
        nsec.magenta(),
        " ".repeat(nsec_right),
        flower.magenta(),
        vertical.magenta()
    );
    println!("{}", nsec_line);

    // Empty line with flowers
    println!(
        "{}{}{}{}{}",
        vertical.magenta(),
        flower.magenta(),
        " ".repeat(width - 5),
        flower.magenta(),
        vertical.magenta()
    );

    // Bottom border with In Identity We Trust
    // Format: same as top - corner + padding + " " + ornament + " " + text + " " + ornament + " " + padding + corner
    // Spaces: 5 total
    // Total: padding + right_padding + text_len + 9
    let trust_text = "In Identity We Trust";
    let trust_len = trust_text.len();
    let fixed = 2 + 5 + 2 + trust_len; // 2 corners + 5 spaces + 2 ornaments + text = 9 + text_len
    let total_padding = width - fixed;
    let padding = total_padding / 2;
    let right_padding = total_padding - padding;
    let bottom_line = format!(
        "{}{} {} {} {} {}{}",
        corner_bl,
        horizontal.repeat(padding),
        ornament,
        trust_text,
        ornament,
        horizontal.repeat(right_padding),
        corner_br
    );
    println!("{}", bottom_line.magenta());
}

/// Generates a random secret key and prints it as an nsec.
fn gensec(args: &Vec<String>) {
    // 1 Match the argument name.
    match args[1].to_lowercase().as_str() {
        // 1.a Command is 'gensec'.
        "gensec" => {
            // 1.a.1 Generate a random nsec.
            let nsec = {
                // 1.a.1.1 Generate a random secret key.
                let secret_key_bytes = generate_secret();

                // 1.a.1.2 Convert the secret key to an nsec.
                match secret_key_bytes.to_nsec() {
                    // 1.a.1.2.a Success.
                    Some(nsec) => nsec,

                    // 1.a.2.b This not possible.
                    None => {
                        println!("{}", "Failed to convert secret key to nsec.".red());
                        return;
                    }
                }
            };

            // 1.a.2 Print the nsec in a decorative frame.
            print_nsec_frame(&nsec);

            // 1.a.3 Drop the nsec.
            drop(nsec);

            // 1.a.4 Return.
            return;
        }

        // 1.b Command is invalid.
        _ => print_correct_usage(),
    }
}

/// Prints genesis params as pretty JSON (random engine key + genesis payload P2TR address).
fn genesis(args: &Vec<String>) {
    // 1 Match the argument name.
    match args[1].to_lowercase().as_str() {
        // 1.a Command is 'genesis'.
        "genesis" => {
            // 1.a.1 Parse chain.
            let chain = match args[2].to_lowercase().as_str() {
                "signet" => Chain::Signet,
                "mainnet" => Chain::Mainnet,
                "testbed" => Chain::Testbed,
                _ => {
                    eprintln!("{}", "Invalid <chain>.".red());
                    return;
                }
            };

            // 1.a.2 Generate a random secret (same source as `gensec`).
            let secret_key_bytes = generate_secret();

            // 1.a.2.1 Encode the secret key bytes to a hex string.
            let secret_key_hex = hex::encode(secret_key_bytes);

            // 1.a.2.2 Convert the secret key bytes to an nsec.
            let nsec = match secret_key_bytes.to_nsec() {
                Some(nsec) => nsec,
                None => {
                    eprintln!("{}", "Failed to encode secret key as nsec.".red());
                    return;
                }
            };

            // 1.a.2.3 Create a key holder from the secret key bytes.
            let key_holder = match KeyHolder::new(secret_key_bytes) {
                Some(k) => k,
                None => {
                    eprintln!("{}", "Generated secret key was invalid.".red());
                    return;
                }
            };

            // 1.a.2.4 Get the public key bytes from the key holder.
            let public_key_bytes = key_holder.secp_public_key_bytes();

            // 1.a.2.5 Encode the public key bytes to a hex string.
            let public_key_hex = hex::encode(public_key_bytes);

            // 1.a.2.6 Convert the public key bytes to an npub.
            let npub = match public_key_bytes.to_npub() {
                Some(npub) => npub,
                None => {
                    eprintln!("{}", "Failed to encode public key as npub.".red());
                    return;
                }
            };

            // 1.a.2.7 Get the genesis payload address.
            let payload_address = match genesis_payload_address(chain, public_key_bytes) {
                Some(addr) => addr,
                None => {
                    eprintln!("{}", "Failed to derive genesis payload address.".red());
                    return;
                }
            };

            // 1.a.2.8 Build the genesis params object.
            let genesis_params = json!({
                "engine": {
                    "secret_key": {
                        "hex": secret_key_hex,
                        "nsec": nsec,
                    },
                    "public_key": {
                        "hex": public_key_hex,
                        "npub": npub,
                    },
                },
                "genesis_payload_address": payload_address,
            });

            // 1.a.2.9 Print pretty JSON.
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "genesis_params": genesis_params
                }))
                .expect("Failed to serialize genesis params.")
            );
        }

        // 1.b Command is invalid.
        _ => print_correct_usage(),
    }
}

/// Runs the appropriate mode based on the arguments.
fn run(args: &Vec<String>) {
    // 1 Parse resource mode.
    let resource_mode = match args[1].to_lowercase().as_str() {
        "pruned" => ResourceMode::Pruned,
        "archival" => ResourceMode::Archival,
        _ => {
            println!("{}", "Invalid <resource mode>.".red());
            return;
        }
    };

    // 2 Parse chain.
    let chain = match args[2].to_lowercase().as_str() {
        "signet" => Chain::Signet,
        "mainnet" => Chain::Mainnet,
        "testbed" => {
            println!("{}", "Testbed is for local tests only (./tests/).".red());
            return;
        }
        _ => {
            println!("{}", "Invalid <chain>.".red());
            return;
        }
    };

    // 3 Parse operating kind.
    let operating_kind = match args[3].to_lowercase().as_str() {
        "node" => OperatingKind::Node,
        "engine" => OperatingKind::Engine,
        _ => {
            println!("{}", "Invalid <kind>.".red());
            return;
        }
    };

    // 4 Parse RPC.
    let rpc_holder =
        BitcoinRPCHolder::new(args[4].to_owned(), args[5].to_owned(), args[6].to_owned());

    // 5 Parse sync mode.
    let sync_mode = match args[7].to_lowercase().as_str() {
        "true" | "yes" | "1" => SyncMode::ConfirmedOnly,
        "false" | "no" | "0" => SyncMode::InFlight,
        _ => {
            println!("{}", "Invalid <syncinflight?>.".red());
            return;
        }
    };

    // 6 Parse key holder.
    let key_holder = {
        // 6.1 Print the prompt.
        println!("{}", "Enter nsec:".magenta());

        // 6.2 Parse the secret key.
        let secret_key: [u8; 32] = {
            // 6.2.1 Initialize the secret key bytes.
            let mut secret_key_bytes = [0xffu8; 32];

            //
            // DANGER ZONE BEGIN: reading private key from stdin.
            //
            {
                // 6.2.2 Read the input from stdin.
                let stdin = std::io::stdin();

                // 6.2.3 Get the handle.
                let handle = stdin.lock();

                // 6.2.4 Drop stdin.
                drop(stdin);

                // 6.2.5 Parse the input.
                for line in handle.lines() {
                    // 6.2.5.1 Unwrap the line.
                    let line = line.unwrap();

                    // 6.2.5.2 Parse the parts.
                    let parts: Vec<&str> = line.trim().split_whitespace().collect();

                    // 6.2.5.3 Check if the parts length is valid.
                    if parts.len() != 1 {
                        println!("{}", "Invalid nsec.".yellow());
                    }

                    // 6.2.5.4 Parse the nsec.
                    let nsec: String = parts[0].to_owned();

                    // 6.2.5.5 Drop the parts.
                    drop(parts);

                    // 6.2.5.6 Convert the nsec to a secret key.
                    secret_key_bytes = match nsec.as_str().from_nsec() {
                        Some(secret_key) => secret_key,
                        None => {
                            eprintln!("{}", "Invalid nsec.".red());
                            return;
                        }
                    };

                    // 6.2.5.7 Drop the nsec.
                    drop(nsec);

                    // 6.2.5.8 Break the loop.
                    break;
                }
            }
            //
            // DANGER ZONE END: reading private key from stdin.
            //

            // 6.2.4 Return the secret key bytes.
            secret_key_bytes
        };

        // 6.3 Create the key holder from the secret key bytes.
        let key_holder = match KeyHolder::new(secret_key) {
            Some(key_holder) => key_holder,
            None => {
                eprintln!("{}", "Invalid nsec.".red());
                return;
            }
        };

        // 6.4 Return the key holder.
        key_holder
    };

    // 7 Run the runner
    runner::run(
        resource_mode,
        chain,
        operating_kind,
        rpc_holder,
        sync_mode,
        key_holder,
    );
}

/// Prints the correct usage of the command.
fn print_correct_usage() {
    eprintln!(
        "{}",
        format!(
            "Usage:\n  gensec\n  genesis <mainnet|signet|testbed>\n  <mode> <chain> <kind> <bitcoin-rpc-url> <bitcoin-rpc-user> <bitcoin-rpc-password> <syncinflight?>"
        )
        .red()
    );
}

/// P2TR address for the genesis inscription payload built from the given engine x-only public key.
fn genesis_payload_address(chain: Chain, engine_key: [u8; 32]) -> Option<String> {
    // 1 Get the payload bytes.
    let payload_bytes = baked::GENESIS_INSCRIPTION.to_vec();

    // 2 Construct the genesis payload without location.
    let genesis_payload_without_location = Payload::new(engine_key, payload_bytes, None);

    // 3 Get the taproot for the genesis payload.
    let genesis_payload_taproot = genesis_payload_without_location.taproot()?;

    // 4 Get the tweaked key for the genesis payload.
    let genesis_payload_taproot_key: [u8; 32] =
        genesis_payload_taproot.tweaked_key()?.serialize_xonly();

    // 5 Encode the tweaked key into an address.
    encode_p2tr(chain, genesis_payload_taproot_key)
}
