use colored::Colorize;
use cube::{
    communicative::rpc::bitcoin_rpc::bitcoin_rpc_holder::BitcoinRPCHolder,
    operative::{
        mode::{coordinator::coordinator, node::node, operator::operator},
        Chain, OperatingMode,
    },
    transmutative::key::{FromNostrKeyStr, KeyHolder},
};
use std::{env, io::BufRead};

fn main() {
    let args: Vec<String> = env::args().collect();

    // Ensure at least 6 arguments: program name, network, mode, rpc-url, rpc-user, rpc-password.
    if args.len() < 6 {
        eprintln!(
            "{}",
            format!(
                "Usage: {} <network> <mode> <rpc-url> <rpc-user> <rpc-password>",
                args[0]
            )
            .red()
        );
        return;
    }

    // Network arg
    let network = match args[1].to_lowercase().as_str() {
        "signet" => Chain::Signet,
        "mainnet" => Chain::Mainnet,
        _ => {
            println!("{}", "Invalid <network>.".red());
            return;
        }
    };

    // Operating mode arg
    let operating_mode = match args[2].to_lowercase().as_str() {
        "node" => OperatingMode::Node,
        "engine" => OperatingMode::Operator,
        "coordinator" => OperatingMode::Coordinator,
        _ => {
            println!("{}", "Invalid <mode>.".red());
            return;
        }
    };

    // RPC args
    let rpc_holder =
        BitcoinRPCHolder::new(args[3].to_owned(), args[4].to_owned(), args[5].to_owned());

    // Key holder
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

    // Match run
    match operating_mode {
        OperatingMode::Node => node::run(key_holder, network, rpc_holder),
        OperatingMode::Operator => operator::run(key_holder, network, rpc_holder),
        OperatingMode::Coordinator => coordinator::run(key_holder, network, rpc_holder),
    }
}
