/// Caller can be the account key itself or another contract.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Caller {
    Account([u8; 32]),
    Contract([u8; 32]),
}

impl Caller {
    /// Creates a new caller from an account key.
    pub fn new_account(account_key: [u8; 32]) -> Self {
        Self::Account(account_key)
    }

    /// Creates a new caller from a contract id.
    pub fn new_contract(contract_id: [u8; 32]) -> Self {
        Self::Contract(contract_id)
    }

    /// Returns true if the caller is an account.
    pub fn is_account(&self) -> bool {
        matches!(self, Self::Account(_))
    }

    /// Returns true if the caller is a contract.
    pub fn is_contract(&self) -> bool {
        matches!(self, Self::Contract(_))
    }

    /// Returns the caller id.
    pub fn caller_id(&self) -> [u8; 32] {
        match self {
            Self::Account(account_key) => *account_key,
            Self::Contract(contract_id) => *contract_id,
        }
    }
}
