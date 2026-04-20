use crate::communicative::rpc::bitcoin_rpc::bitcoin_rpc_error::{
    BitcoinRPCBroadcastRawTransactionError, BitcoinRPCGetChainTipError, BitcoinRPCRetrieveBlockError,
    BitcoinRPCValidateRPCError,
};
use crate::communicative::rpc::bitcoin_rpc::bitcoin_rpc_holder::BitcoinRPCHolder;
use crate::operative::run_args::chain::Chain;
use bitcoin::{Block, BlockHash, Transaction, Txid};
use bitcoincore_rpc::{json::GetBlockchainInfoResult, Auth, Client, RpcApi};

/// Validates the Bitcoin RPC.
pub fn validate_rpc(
    rpc_holder: &BitcoinRPCHolder,
    chain: Chain,
) -> Result<(), BitcoinRPCValidateRPCError> {
    let rpc_url = rpc_holder.url();
    let rpc_user = rpc_holder.user();
    let rpc_password = rpc_holder.password();

    // Create RPC client.
    let rpc_client = match Client::new(&rpc_url, Auth::UserPass(rpc_user, rpc_password)) {
        Ok(client) => client,
        Err(err) => return Err(BitcoinRPCValidateRPCError::RPCErr(err)),
    };

    // Get blockchain info.
    let blockchain_info: GetBlockchainInfoResult = match rpc_client.get_blockchain_info() {
        Ok(result) => result,
        Err(err) => return Err(BitcoinRPCValidateRPCError::RPCErr(err)),
    };

    // Validate chain.
    match blockchain_info.chain {
        bitcoin::network::Network::Bitcoin => {
            if chain != Chain::Mainnet {
                return Err(BitcoinRPCValidateRPCError::WrongChain);
            }
        }
        bitcoin::network::Network::Signet => {
            if chain != Chain::Signet {
                return Err(BitcoinRPCValidateRPCError::WrongChain);
            }
        }
        _ => return Err(BitcoinRPCValidateRPCError::WrongChain),
    };

    // Validate sync status.
    if blockchain_info.initial_block_download {
        return Err(BitcoinRPCValidateRPCError::NotSynced);
    }

    // Return success.
    Ok(())
}

/// Returns the chain tip (latest block height).
pub fn get_chain_tip(
    rpc_holder: &BitcoinRPCHolder,
) -> Result<(u64, bool), BitcoinRPCGetChainTipError> {
    let rpc_url = rpc_holder.url();
    let rpc_user = rpc_holder.user();
    let rpc_password = rpc_holder.password();

    // Create RPC client.
    let rpc_client = match Client::new(&rpc_url, Auth::UserPass(rpc_user, rpc_password)) {
        Ok(client) => client,
        Err(err) => return Err(BitcoinRPCGetChainTipError::RPCErr(err)),
    };

    // Get blockchain info.
    let blockchain_info: GetBlockchainInfoResult = match rpc_client.get_blockchain_info() {
        Ok(result) => result,
        Err(err) => return Err(BitcoinRPCGetChainTipError::RPCErr(err)),
    };

    // Check if the Bitcoin node is fully synced.
    let is_synced = !blockchain_info.initial_block_download;

    // Get chain height.
    let chain_height = blockchain_info.blocks;

    // Return chain height.
    Ok((chain_height, is_synced))
}

/// Returns the block at the given height.
pub fn retrieve_block(
    rpc_holder: &BitcoinRPCHolder,
    height: u64,
) -> Result<bitcoin::blockdata::block::Block, BitcoinRPCRetrieveBlockError> {
    let rpc_url = rpc_holder.url();
    let rpc_user = rpc_holder.user();
    let rpc_password = rpc_holder.password();

    // Create RPC client.
    let rpc_client = match Client::new(&rpc_url, Auth::UserPass(rpc_user, rpc_password)) {
        Ok(client) => client,
        Err(err) => return Err(BitcoinRPCRetrieveBlockError::RPCErr(err)),
    };

    // Get block hash.
    let block_hash: BlockHash = match rpc_client.get_block_hash(height) {
        Ok(block_hash) => block_hash,
        Err(err) => return Err(BitcoinRPCRetrieveBlockError::RPCErr(err)),
    };

    // Get block.
    let block: Block = match rpc_client.get_block(&block_hash) {
        Ok(block) => block,
        Err(err) => return Err(BitcoinRPCRetrieveBlockError::RPCErr(err)),
    };

    // Return block.
    Ok(block)
}

/// Broadcasts a raw transaction hex and returns its txid.
pub fn broadcast_raw_transaction(
    rpc_holder: &BitcoinRPCHolder,
    raw_transaction_hex: &str,
) -> Result<Txid, BitcoinRPCBroadcastRawTransactionError> {
    let rpc_url = rpc_holder.url();
    let rpc_user = rpc_holder.user();
    let rpc_password = rpc_holder.password();

    // Create RPC client.
    let rpc_client = match Client::new(&rpc_url, Auth::UserPass(rpc_user, rpc_password)) {
        Ok(client) => client,
        Err(err) => return Err(BitcoinRPCBroadcastRawTransactionError::RPCErr(err)),
    };

    // Decode raw transaction hex into a bitcoin::Transaction.
    let raw_bytes = match hex::decode(raw_transaction_hex) {
        Ok(raw_bytes) => raw_bytes,
        Err(err) => return Err(BitcoinRPCBroadcastRawTransactionError::HexErr(err)),
    };
    let transaction: Transaction = match bitcoin::consensus::encode::deserialize(&raw_bytes) {
        Ok(transaction) => transaction,
        Err(err) => return Err(BitcoinRPCBroadcastRawTransactionError::DecodeErr(err)),
    };

    // Broadcast the transaction.
    match rpc_client.send_raw_transaction(&transaction) {
        Ok(txid) => Ok(txid),
        Err(err) => Err(BitcoinRPCBroadcastRawTransactionError::RPCErr(err)),
    }
}
