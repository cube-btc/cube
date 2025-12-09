pub mod mode;
pub mod sync;

/// Chain type.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Chain {
    // For local tests (./tests/) involving db operations.
    Testbed,
    // For signet.
    Signet,
    // For mainnet.
    Mainnet,
}

impl ToString for Chain {
    fn to_string(&self) -> String {
        match self {
            Chain::Testbed => "testbed".to_string(),
            Chain::Signet => "signet".to_string(),
            Chain::Mainnet => "mainnet".to_string(),
        }
    }
}

/// Operating kind type.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OperatingKind {
    Node,
    Operator,
    Coordinator,
}

/// Operating mode type.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OperatingMode {
    Pruned,
    Archival,
}
