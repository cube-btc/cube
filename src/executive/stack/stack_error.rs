use crate::inscriptive::{
    coin_manager::errors::balance_update_errors::{
        CMAccountBalanceUpError, CMContractBalanceDownError, CMContractBalanceUpError,
    },
    coin_manager::errors::shadow_alloc_errors::{
        CMContractShadowAllocAccountError, CMContractShadowDeallocAccountError,
    },
    coin_manager::errors::shadow_update_errors::{
        CMShadowDownAllError, CMShadowDownError, CMShadowUpAllError, CMShadowUpError,
    },
    state_manager::errors::insert_update_state_error::SMInsertUpdateStateError,
};

/// The call error.
#[derive(Debug, Clone)]
pub enum CallError {
    /// The contract id is invalid.
    InvalidContractId,
    /// The method index is invalid.
    InvalidMethodIndex,
    /// The arguments count is invalid.
    InvalidArgumentsCount,
}

/// The BLS error.
#[derive(Debug, Clone)]
pub enum BLSError {
    /// The BLS public key is invalid.
    InvalidBLSPublicKeyBytes,
    /// The BLS message is invalid.
    InvalidBLSMessageBytes,
    /// The BLS signature is invalid.
    InvalidBLSSignatureBytes,
}

/// The Schnorr error.
#[derive(Debug, Clone)]
pub enum SchnorrError {
    /// The Schnorr public key is invalid.
    InvalidSchnorrPublicKeyBytes,
    /// The Schnorr message is invalid.
    InvalidSchnorrMessageBytes,
    /// The Schnorr signature is invalid.
    InvalidSchnorrSignatureBytes,
}

/// The secp error.
#[derive(Debug, Clone)]
pub enum SecpError {
    /// The secp scalar is invalid.
    InvalidSecpScalar,
    /// The secp point is invalid.
    InvalidSecpPoint,
}

/// The stack uint error.
#[derive(Debug, Clone, Copy)]
pub enum StackUintError {
    /// The stack uint max overflow error.
    StackUintMaxOverflowError,
    /// The stack uint conversion error.
    StackUintConversionError,
}

/// The ops budget error.
#[derive(Debug, Clone)]
pub enum OpsBudgetError {
    /// The internal ops budget exceeded error.
    InternalOpsBudgetExceeded,
    /// The external ops limit exceeded error.
    ExternalOpsLimitExceeded,
}

/// The storage error.
#[derive(Debug, Clone)]
pub enum StorageError {
    /// The invalid storage key length error.
    InvalidStorageKeyLength(u8),
    /// The invalid storage value length error.
    InvalidStorageValueLength(u8),
    /// The state holder insert update value error.
    StateManagerInsertUpdateStateError(SMInsertUpdateStateError),
}

/// The memory error.
#[derive(Debug, Clone)]
pub enum MemoryError {
    /// The invalid memory key length error.
    InvalidMemoryKeyLength(u8),
    /// The invalid memory value length error.
    InvalidMemoryValueLength(u8),
    /// The memory size limit exceeded error.
    ContractMemorySizeLimitExceeded,
}

/// The mandatory error.
#[derive(Debug, Clone)]
pub enum MandatoryError {
    /// The mandatory equal verify error.
    MandatoryEqualVerifyError,
    /// The mandatory verify error.
    MandatoryVerifyError,
}

/// The coin balance get error.
#[derive(Debug, Clone)]
pub enum CoinBalanceGetError {
    /// The kind tier is invalid.
    InvalidKindTier(usize),
    /// The account key is invalid.
    InvalidAccountKeyBytes(Vec<u8>),
    /// The contract id is invalid.
    InvalidContractIdBytes(Vec<u8>),
    /// The unable to get account balance error.
    UnableToGetAccountBalance([u8; 32]),
    /// The unable to get contract balance error.
    UnableToGetContractBalance([u8; 32]),
}

/// The coin transfer error.
#[derive(Debug, Clone)]
pub enum CoinTransferError {
    /// The kind tier is invalid.
    InvalidKindTier(usize),
    /// The account key is invalid.
    InvalidAccountKeyBytes(Vec<u8>),
    /// The contract id is invalid.
    InvalidContractIdBytes(Vec<u8>),
    /// The amount is invalid.
    InvalidAmountBytes(Vec<u8>),
    /// The contract balance down error.
    ContractBalanceDownError([u8; 32], CMContractBalanceDownError),
    /// The account balance up error.
    AccountBalanceUpError([u8; 32], CMAccountBalanceUpError),
    /// The contract balance up error.
    ContractBalanceUpError([u8; 32], CMContractBalanceUpError),
}

/// The shadow ops error.
#[derive(Debug, Clone)]
pub enum ShadowOpsError {
    /// The account key is invalid.
    InvalidAccountKeyBytes(Vec<u8>),
    /// The amount is invalid.
    InvalidAmountBytes(Vec<u8>),
    /// The shadow alloc error.
    ShadowAllocError(CMContractShadowAllocAccountError),
    /// The shadow dealloc error.
    ShadowDeallocError(CMContractShadowDeallocAccountError),
    /// The shadow alloc up error.
    ShadowAllocUpError(CMShadowUpError),
    /// The shadow alloc down error.
    ShadowAllocDownError(CMShadowDownError),
    /// The shadow alloc up all error.
    ShadowAllocUpAllError(CMShadowUpAllError),
    /// The shadow alloc down all error.
    ShadowAllocDownAllError(CMShadowDownAllError),
    /// The account key has no allocation.
    AccountKeyHasNoAllocation([u8; 32]),
}

/// The stack error.
#[derive(Debug, Clone)]
pub enum StackError {
    /// The stack is empty.
    EmptyStack,
    /// The stack item is too large.
    StackItemTooLarge,
    /// The stack is too large.
    StackTooLarge,
    /// The pick index is out of bounds.
    PickIndexError(u32),
    /// The remove index is out of bounds.
    RemoveIndexError(u32),
    /// The mandatory error.
    MandatoryError(MandatoryError),
    /// The memory error.
    MemoryError(MemoryError),
    /// The storage error.
    StorageError(StorageError),
    /// The ops budget error.
    OpsBudgetError(OpsBudgetError),
    /// The stack uint error.
    StackUintError(StackUintError),
    /// The fail error.
    FailError,
    /// The OP_ELSE encountered with preceding OP_ELSE error.
    OPElseEncounteredWithPrecedingAnotherOPElse,
    // OP_ELSE encountered without preceding flow encounter error.
    OPElseEncounteredWithoutPrecedingFlowEncounter,
    // Reserved opcode encountered error.
    ReservedOpcodeEncounteredError,
    // Split index error.
    SplitIndexError,
    // Blake2b error.
    BLAKE2bVarOutputSizeError,
    // Blake2s error.
    BLAKE2sVarOutputSizeError,
    // Secp error.
    SecpError(SecpError),
    // BLS error.
    BLSError(BLSError),
    // Schnorr error.
    SchnorrError(SchnorrError),
    // Call error.
    CallError(CallError),
    // Key conversion error.
    Key32BytesConversionError,
    /// The coin balance get error.
    CoinBalanceGetError(CoinBalanceGetError),
    /// The coin transfer error.
    CoinTransferError(CoinTransferError),
    /// The shadow ops error.
    ShadowOpsError(ShadowOpsError),
}
