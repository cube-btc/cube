use crate::inscriptive::baked;
use crate::operative::run_args::chain::Chain;

/// Prints the baked well-known engine public key (hex) for the active chain.
pub fn engine_command(chain: Chain) {
    let key = match chain {
        Chain::Mainnet => baked::MAINNET_ENGINE_PUBLIC_KEY,
        Chain::Signet | Chain::Testbed => baked::SIGNET_ENGINE_PUBLIC_KEY,
    };
    println!("{}", hex::encode(key));
}
