/// Errors that can occur when encoding an `OpsPrice` into an APE bit vector.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OpsPriceAPEEncodeError {
    /// `ops_price_ppm` is strictly less than `base_ops_price`.
    OpsPriceBelowBaseOpsPrice {
        ops_price_ppm: u64,
        base_ops_price: u64,
    },
}
