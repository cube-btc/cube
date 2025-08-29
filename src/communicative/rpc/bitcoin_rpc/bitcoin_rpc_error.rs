use std::fmt;

#[derive(Debug)]
pub enum BitcoinRPCValidateRPCError {
    WrongChain,
    NotSynced,
    RPCErr(bitcoincore_rpc::Error),
}

#[derive(Debug)]
pub enum BitcoinRPCGetChainTipError {
    RPCErr(bitcoincore_rpc::Error),
}

#[derive(Debug)]
pub enum BitcoinRPCRetrieveBlockError {
    RPCErr(bitcoincore_rpc::Error),
}

impl fmt::Display for BitcoinRPCValidateRPCError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BitcoinRPCValidateRPCError::WrongChain => write!(f, "Wrong chain."),
            BitcoinRPCValidateRPCError::NotSynced => write!(f, "Node is not fully synced yet."),
            BitcoinRPCValidateRPCError::RPCErr(err) => write!(f, "RPC error: {}", err),
        }
    }
}

impl fmt::Display for BitcoinRPCGetChainTipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BitcoinRPCGetChainTipError::RPCErr(err) => write!(f, "RPC error: {}", err),
        }
    }
}

impl fmt::Display for BitcoinRPCRetrieveBlockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BitcoinRPCRetrieveBlockError::RPCErr(err) => write!(f, "RPC error: {}", err),
        }
    }
}
