use crate::constructive::core_types::ops_price::ops_price::OpsPrice;

impl OpsPrice {
    /// Encodes this `OpsPrice` as Structural Byte-scope Encoding (SBE) bytes.
    ///
    /// Layout: four bytes, little-endian `u32` `ops_price_ppm`.
    pub fn encode_sbe(&self) -> [u8; 4] {
        self.ops_price_ppm.to_le_bytes()
    }
}
