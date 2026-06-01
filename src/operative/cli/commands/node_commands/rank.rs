use crate::inscriptive::registry::registry::REGISTRY;
use colored::Colorize;

/// Prints the registry call-counter rank for a permanently stored account key.
pub async fn account_rank_command(registry: &REGISTRY, account_key: [u8; 32]) {
    let rank = {
        let r = registry.lock().await;
        r.get_rank_by_account_key(account_key)
    };
    match rank {
        Some(rank) => println!("{}", rank),
        None => eprintln!(
            "{}",
            "No rank for this account key in the registry (not registered on disk)."
                .yellow()
        ),
    }
}

/// Prints the registry call-counter rank for a permanently stored contract id.
pub async fn contract_rank_command(registry: &REGISTRY, contract_id: [u8; 32]) {
    let rank = {
        let r = registry.lock().await;
        r.get_rank_by_contract_id(contract_id)
    };
    match rank {
        Some(rank) => println!("{}", rank),
        None => eprintln!(
            "{}",
            "No rank for this contract id in the registry (not registered on disk)."
                .yellow()
        ),
    }
}
