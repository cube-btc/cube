use crate::constructive::entity::account::account::account::Account;

/// A type alias for a vector of bytes.
type Bytes = Vec<u8>;

impl Account {
    /// Structural Byte-scope Encoding (SBE) encoding for `Account`.
    ///
    /// This function encodes an `Account` in a non-compact, byte-scope format analogous to
    /// [`RootAccount::encode_sbe`](crate::constructive::entity::account::root_account::root_account::RootAccount::encode_sbe):
    /// a leading variant discriminant byte, then a fixed layout per variant.
    ///
    /// Layout:
    /// - `0x00` — `UnregisteredAccount`: 32-byte Schnorr account key (`account_key_to_be_registered`).
    /// - `0x01` — `RegisteredAccount`: 32-byte Schnorr account key, then 8-byte little-endian `registery_index`.
    pub fn encode_sbe(&self) -> Bytes {
        // 1 Initialize the byte vector.
        let mut bytes = Bytes::new();

        // 2 Match on the `Account` variant.
        match self {
            // 2.a The `Account` is an `UnregisteredAccount`.
            Account::UnregisteredAccount(unregistered_account) => {
                // 2.a.1 Push `0x00` to indicate an `UnregisteredAccount`.
                bytes.push(0x00);

                // 2.a.2 Encode the Schnorr account key to be registered.
                bytes.extend_from_slice(&unregistered_account.account_key_to_be_registered);
            }

            // 2.b The `Account` is a `RegisteredAccount`.
            Account::RegisteredAccount(registered_account) => {
                // 2.b.1 Push `0x01` to indicate a `RegisteredAccount`.
                bytes.push(0x01);

                // 2.b.2 Encode the Schnorr account key.
                bytes.extend_from_slice(&registered_account.account_key);

                // 2.b.3 Encode the registery index as little-endian `u64`.
                bytes.extend_from_slice(&registered_account.registery_index.to_le_bytes());
            }
        }

        // 3 Return the bytes.
        bytes
    }
}
