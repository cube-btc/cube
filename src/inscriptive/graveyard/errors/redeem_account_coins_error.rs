/// Account key.
type AccountKey = [u8; 32];

/// Satoshi redemption amount.
type SatoshiRedemptionAmount = u64;

/// Minimum redemption amount.
type MinimumRedemptionAmount = u64;

/// Errors associated with redeeming an account.
#[derive(Debug, Clone)]
pub enum GraveyardRedeemAccountCoinsError {
    AccountCoinsHasJustBeenEphemerallyRedeemed(AccountKey),
    ThisAccountHasJustBeenEphemerallyBurried(AccountKey),
    RedemptionAmountNotFound(AccountKey),
    AccountCoinsHasAlreadyBeenEphemerallyRedeemed(AccountKey),
    RedemptionAmountIsLessThanTheMinimumRedemptionAmount(
        AccountKey,
        SatoshiRedemptionAmount,
        MinimumRedemptionAmount,
    ),
}
