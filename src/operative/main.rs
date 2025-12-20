use colored::Colorize;
use cube::{
    communicative::rpc::bitcoin_rpc::bitcoin_rpc_holder::BitcoinRPCHolder,
    operative::{
        mode::{coordinator::coordinator, node::node, operator::operator},
        Chain, OperatingKind, OperatingMode,
    },
    transmutative::{
        key::{FromNostrKeyStr, KeyHolder, ToNostrKeyStr},
        secp::schnorr::generate_secret,
    },
};
use std::{env, io::BufRead};

fn main() {
    // 1 Parse arguments.
    let args: Vec<String> = env::args().collect();

    // 2 Match the arguments length.
    match args.len() {
        // 2.a Generate a random secret key and print it as an nsec.
        2 => gensec(&args),

        // 2.b Run the appropriate mode based on the arguments.
        7 => run(&args),

        // 2.c Invalid arguments.
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

/// Runs the appropriate mode based on the arguments.
fn run(args: &Vec<String>) {
    // 1 Parse operating mode.
    let operating_mode = match args[1].to_lowercase().as_str() {
        "pruned" => OperatingMode::Pruned,
        "archival" => OperatingMode::Archival,
        _ => {
            println!("{}", "Invalid <mode>.".red());
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
        "engine" => OperatingKind::Operator,
        "coordinator" => OperatingKind::Coordinator,
        _ => {
            println!("{}", "Invalid <kind>.".red());
            return;
        }
    };

    // 4 Parse RPC.
    let rpc_holder =
        BitcoinRPCHolder::new(args[4].to_owned(), args[5].to_owned(), args[6].to_owned());

    // 5 Parse key holder.
    let key_holder = {
        // 5.1 Print the prompt.
        println!("{}", "Enter nsec:".magenta());

        // 5.2 Parse the secret key.
        let secret_key: [u8; 32] = {
            // 5.2.1 Initialize the secret key bytes.
            let mut secret_key_bytes = [0xffu8; 32];

            //
            // DANGER ZONE BEGIN: reading private key from stdin.
            //
            {
                // 5.2.2 Read the input from stdin.
                let stdin = std::io::stdin();

                // 5.2.3 Get the handle.
                let handle = stdin.lock();

                // 5.2.4 Drop stdin.
                drop(stdin);

                // 5.2.5 Parse the input.
                for line in handle.lines() {
                    // 5.2.5.1 Unwrap the line.
                    let line = line.unwrap();

                    // 5.2.5.2 Parse the parts.
                    let parts: Vec<&str> = line.trim().split_whitespace().collect();

                    // 5.2.5.3 Check if the parts length is valid.
                    if parts.len() != 1 {
                        println!("{}", "Invalid nsec.".yellow());
                    }

                    // 5.2.5.4 Parse the nsec.
                    let nsec: String = parts[0].to_owned();

                    // 5.2.5.5 Drop the parts.
                    drop(parts);

                    // 5.2.5.6 Convert the nsec to a secret key.
                    secret_key_bytes = match nsec.as_str().from_nsec() {
                        Some(secret_key) => secret_key,
                        None => {
                            eprintln!("{}", "Invalid nsec.".red());
                            return;
                        }
                    };

                    // 5.2.5.7 Drop the nsec.
                    drop(nsec);

                    // 5.2.5.8 Break the loop.
                    break;
                }
            }
            //
            // DANGER ZONE END: reading private key from stdin.
            //

            // 5.2.4 Return the secret key bytes.
            secret_key_bytes
        };

        // 5.3 Create the key holder from the secret key bytes.
        let key_holder = match KeyHolder::new(secret_key) {
            Some(key_holder) => key_holder,
            None => {
                eprintln!("{}", "Invalid nsec.".red());
                return;
            }
        };

        // 5.4 Return the key holder.
        key_holder
    };

    // 6 Run the appropriate mode.
    match operating_kind {
        // 6.1 Run as a node.
        OperatingKind::Node => node::run(key_holder, chain, rpc_holder, operating_mode),

        // 6.2 Run as an operator.
        OperatingKind::Operator => operator::run(key_holder, chain, rpc_holder, operating_mode),

        // 6.3 Run as a coordinator.
        OperatingKind::Coordinator => {
            coordinator::run(key_holder, chain, rpc_holder, operating_mode)
        }
    }
}

/// Prints the correct usage of the command.
fn print_correct_usage() {
    eprintln!(
        "{}",
        format!(
            "Usage: <mode> <chain> <kind> <bitcoin-rpc-url> <bitcoin-rpc-user> <bitcoin-rpc-password>"
        )
        .red()
    );
    return;
}
