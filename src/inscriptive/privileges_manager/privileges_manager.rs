use crate::inscriptive::privileges_manager::bodies::account_body::account_body::PMAccountBody;
use crate::inscriptive::privileges_manager::bodies::contract_body::contract_body::PMContractBody;
use crate::inscriptive::privileges_manager::delta::delta::PMDelta;
use std::collections::HashMap;

/// Account key.
type AccountKey = [u8; 32];

/// Contract ID.
type ContractId = [u8; 32];

/// A struct for managing the privileges and fee management resources of accounts and contracts.
#[allow(dead_code)]
pub struct PrivilegesManager {
    // In-memory accounts.
    in_memory_accounts: HashMap<AccountKey, PMAccountBody>,

    // In-memory contracts.
    in_memory_contracts: HashMap<ContractId, PMContractBody>,

    // On-disk accounts.
    on_disk_accounts: sled::Db,

    // On-disk contracts.
    on_disk_contracts: sled::Db,

    // State differences to be applied.
    delta: PMDelta,

    // Backup of state differences in case of rollback.
    backup_of_delta: PMDelta,
}
