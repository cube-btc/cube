/// Baked, initial base ops price parameter.
/// Key prefix: 0x02
/// ------------------------------------------------------------

pub const BASE_OPS_PRICE_KEY: [u8; 2] = [0x02, 0x00];
pub const BASE_OPS_PRICE_PPM_VALUE: u64 = 100_000;

// NOTE: Base ops price is expressed in PPM (parts per million) similar to that of liquidity fees.
// This is for better precision in fee calculations.
// Base ops price in PPM is bake-coded as 100_000, which means 10 ops costs 1 coin == 1 satoshi.
