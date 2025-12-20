#[cfg(test)]
mod bls_test {
    use cube::transmutative::bls::{
        agg::bls_aggregate,
        key::{
            bls_secret_key_bytes_to_bls_secret_key, bls_secret_key_to_bls_public_key,
            secp_secret_key_bytes_to_bls_secret_key_bytes, BLSPublicKey, BLSSecretKey,
        },
        sign::bls_sign,
        verify::{bls_verify, bls_verify_aggregate},
    };

    #[test]
    fn test_bls_public_key() -> Result<(), String> {
        // Load known hex bytes (instead of generating a new random secret key like in the previous example)
        let secret_key_bytes = [
            0x32, 0x83, 0x88, 0xaf, 0xf0, 0xd4, 0xa5, 0xb7, 0xdc, 0x92, 0x05, 0xab, 0xd3, 0x74,
            0xe7, 0xe9, 0x8f, 0x3c, 0xd9, 0xf3, 0x41, 0x8e, 0xdb, 0x4e, 0xaf, 0xda, 0x5f, 0xb1,
            0x64, 0x73, 0xd2, 0x16,
        ];

        let bls_secret_key_bytes: [u8; 48] =
            secp_secret_key_bytes_to_bls_secret_key_bytes(&secret_key_bytes);

        // Convert the secret key to a BLS public key.
        let public_key: [u8; 48] = bls_secret_key_to_bls_public_key(
            bls_secret_key_bytes_to_bls_secret_key(bls_secret_key_bytes),
        )
        .unwrap()
        .try_into()
        .unwrap();

        // Convert the expected public key to a BLS public key.
        let expected_public_key: [u8; 48] = hex::decode("8c68fce91a1e95147e15947de3f2d82147e83ead638624784d9d4629c142a63135822fdaa5ec70fbf1d63d724bc26b98")
            .unwrap()
            .try_into()
            .unwrap();

        // Check that the public key is correct.
        assert_eq!(public_key, expected_public_key);

        Ok(())
    }

    #[test]
    fn test_bls() -> Result<(), String> {
        // Generate secret keys.
        let secret_key_bytes_1: [u8; 32] =
            hex::decode("5198e1eabd745dd9ca8a7dffbab9b1055d4e110eecb24bfb02231348c70bc248")
                .unwrap()
                .try_into()
                .unwrap();
        let secret_key_bytes_2: [u8; 32] =
            hex::decode("7e70d9454d1db2af2b731487cbcf32e0d392bcac74113627af96f1931df07e2b")
                .unwrap()
                .try_into()
                .unwrap();
        let secret_key_bytes_3: [u8; 32] =
            hex::decode("3b3e17014556fd7ec3e616d7f3ab7759ebd592fc931752be95a7bb1fd0d6e476")
                .unwrap()
                .try_into()
                .unwrap();

        let bls_secret_key_bytes_1: [u8; 48] =
            secp_secret_key_bytes_to_bls_secret_key_bytes(&secret_key_bytes_1);
        let bls_secret_key_bytes_2: [u8; 48] =
            secp_secret_key_bytes_to_bls_secret_key_bytes(&secret_key_bytes_2);
        let bls_secret_key_bytes_3: [u8; 48] =
            secp_secret_key_bytes_to_bls_secret_key_bytes(&secret_key_bytes_3);

        // Convert the secret keys to BLS secret keys.
        let bls_secret_key_1: BLSSecretKey =
            bls_secret_key_bytes_to_bls_secret_key(bls_secret_key_bytes_1);
        let bls_secret_key_2: BLSSecretKey =
            bls_secret_key_bytes_to_bls_secret_key(bls_secret_key_bytes_2);
        let bls_secret_key_3: BLSSecretKey =
            bls_secret_key_bytes_to_bls_secret_key(bls_secret_key_bytes_3);

        // Convert the BLS secret keys to BLS public keys.
        let bls_public_key_1: BLSPublicKey =
            bls_secret_key_to_bls_public_key(bls_secret_key_1).unwrap();
        let bls_public_key_2: BLSPublicKey =
            bls_secret_key_to_bls_public_key(bls_secret_key_2).unwrap();
        let bls_public_key_3: BLSPublicKey =
            bls_secret_key_to_bls_public_key(bls_secret_key_3).unwrap();

        // Message to sign.
        let message_1: [u8; 32] = [0xffu8; 32];
        let message_2: [u8; 32] = [0xfeu8; 32];
        let message_3: [u8; 32] = [0xfdu8; 32];

        // Sign a message with the secret keys.
        let signature_1: [u8; 96] = bls_sign(bls_secret_key_1, message_1);
        let signature_2: [u8; 96] = bls_sign(bls_secret_key_2, message_2);
        let signature_3: [u8; 96] = bls_sign(bls_secret_key_3, message_3);

        // Verify the signatures.
        assert!(bls_verify(&bls_public_key_1, message_1, signature_1));
        assert!(bls_verify(&bls_public_key_2, message_2, signature_2));
        assert!(bls_verify(&bls_public_key_3, message_3, signature_3));

        // Aggregate the signatures.
        let aggregate_signature: [u8; 96] =
            bls_aggregate(vec![signature_1, signature_2, signature_3])
                .unwrap()
                .try_into()
                .unwrap();

        // Verify the aggregate signature.
        assert!(bls_verify_aggregate(
            vec![bls_public_key_1, bls_public_key_2, bls_public_key_3],
            vec![message_1, message_2, message_3],
            aggregate_signature
        ));

        Ok(())
    }
}
