use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// The hierarchy of an account.    
#[derive(Clone, Serialize, Deserialize)]
pub enum AccountHierarchy {
    // A plebeian account.
    Pleb,

    // A resident account.
    Resident,

    // A citizen account.
    Citizen,
}

impl AccountHierarchy {
    /// Creates a new plebeian account hierarchy.
    pub fn new_pleb() -> Self {
        Self::Pleb
    }

    /// Creates a new resident account hierarchy.
    pub fn new_resident() -> Self {
        Self::Resident
    }

    /// Creates a new citizen account hierarchy.
    pub fn new_citizen() -> Self {
        Self::Citizen
    }

    /// Returns whether the account is immutable by checking if it is a citizen.
    pub fn is_immutable(&self) -> bool {
        matches!(self, Self::Citizen)
    }

    /// Returns the account hierarchy from the bytecode.
    pub fn from_bytecode(bytecode: u8) -> Option<Self> {
        match bytecode {
            0x00 => Some(Self::Pleb),
            0x01 => Some(Self::Resident),
            0x02 => Some(Self::Citizen),
            _ => None,
        }
    }

    /// Returns the bytecode of the account hierarchy.
    pub fn to_bytecode(&self) -> u8 {
        match self {
            Self::Pleb => 0x00,
            Self::Resident => 0x01,
            Self::Citizen => 0x02,
        }
    }
}

impl Display for AccountHierarchy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Pleb => "Pleb",
                Self::Resident => "Resident",
                Self::Citizen => "Citizen",
            }
        )
    }
}
