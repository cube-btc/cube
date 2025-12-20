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

/// Generates a random secret key and prints it as an nsec.
fn gensec(args: &Vec<String>) {
    // 1 Match the argument name.
    match args[1].to_lowercase().as_str() {
        // 1.a Command is 'gensec'.
        "gensec" => {
            // 1.a.1 Generate a random secret key.
            let secret_key_bytes = generate_secret();

            // 1.a.2 Secret key as nsec.
            let nsec = match secret_key_bytes.to_nsec() {
                // 1.a.2.a Success.
                Some(nsec) => nsec,

                // 1.a.2.b This not possible.
                None => {
                    println!("{}", "Failed to convert secret key to nsec.".red());
                    return;
                }
            };

            // 1.a.3 Print the nsec.
            println!("{}", nsec.magenta());
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
        println!("{}", "Enter nsec:".magenta());

        let mut secret_key_bytes = [0xffu8; 32];

        let stdin = std::io::stdin();
        let handle = stdin.lock();

        for line in handle.lines() {
            let line = line.unwrap();
            let parts: Vec<&str> = line.trim().split_whitespace().collect();

            if parts.len() != 1 {
                println!("{}", "Invalid nsec.".yellow());
            }

            let nsec = parts[0];

            secret_key_bytes = match nsec.from_nsec() {
                Some(secret_key) => secret_key,
                None => {
                    eprintln!("{}", "Invalid nsec.".red());
                    return;
                }
            };

            break;
        }

        let key_holder = match KeyHolder::new(secret_key_bytes) {
            Some(key_holder) => key_holder,
            None => {
                eprintln!("{}", "Invalid nsec.".red());
                return;
            }
        };

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
