/// Account key.
#[allow(non_camel_case_types)]
type ACCOUNT_KEY = [u8; 32];

/// Contract ID.
#[allow(non_camel_case_types)]
type CONTRACT_ID = [u8; 32];

/// Satoshi amount.
#[allow(non_camel_case_types)]
type SATOSHI_AMOUNT = u64;

/// Sati-satoshi amount.
#[allow(non_camel_case_types)]
type SATI_SATOSHI_AMOUNT = u128;

/// Errors associated with saving account delta changes to the `CoinHolder`.
#[derive(Debug, Clone)]
pub enum CHAccountApplyChangesError {
    OpenTreeError(ACCOUNT_KEY, sled::Error),
    BalanceValueOnDiskInsertionError(ACCOUNT_KEY, SATOSHI_AMOUNT, sled::Error),
    ShadowAllocsSumValueOnDiskInsertionError(ACCOUNT_KEY, SATI_SATOSHI_AMOUNT, sled::Error),
    UnableToGetPermanentAccountBody(ACCOUNT_KEY),
    //TreeValueInsertError(ACCOUNT_KEY, SATOSHI_AMOUNT, sled::Error),
    //UnableToGetAccountBody(ACCOUNT_KEY),
}

/// Errors associated with applying contract delta changes to the `CoinHolder`.
#[derive(Debug, Clone)]
pub enum CHContractApplyChangesError {
    OpenTreeError(CONTRACT_ID, sled::Error),
    BalanceValueOnDiskInsertionError(CONTRACT_ID, SATOSHI_AMOUNT, sled::Error),
    AllocsSumValueOnDiskInsertionError(CONTRACT_ID, SATOSHI_AMOUNT, sled::Error),
    UnableToGetPermanentContractBody(CONTRACT_ID),
    ShadowAllocValueOnDiskInsertionError(
        CONTRACT_ID,
        ACCOUNT_KEY,
        SATI_SATOSHI_AMOUNT,
        sled::Error,
    ),
    InMemoryDeallocAccountError(CONTRACT_ID, ACCOUNT_KEY),
    OnDiskDeallocAccountError(CONTRACT_ID, ACCOUNT_KEY, sled::Error),
}

/// Errors associated with applying account and contract delta changes to the `CoinHolder`.
#[derive(Debug, Clone)]
pub enum CHApplyChangesError {
    AccountApplyChangesError(CHAccountApplyChangesError),
    ContractApplyChangesError(CHContractApplyChangesError),
}
