#[cfg(test)]
mod address_tests {
    use cube::{
        operative::run_args::chain::Chain,
        transmutative::codec::address::{address_to_spk, encode_p2tr, encode_p2wpkh, encode_p2wsh},
    };

    #[test]
    fn encode_p2tr_test() -> Result<(), String> {
        let taproot_key: [u8; 32] =
            hex::decode("9f5e030b7111d8e1e757d31944df5bf0714c94368cc0b499bef0badacb67c2a8")
                .unwrap()
                .try_into()
                .unwrap();

        let signet_address = encode_p2tr(Chain::Signet, taproot_key).unwrap();
        let mainnet_address = encode_p2tr(Chain::Mainnet, taproot_key).unwrap();

        assert_eq!(
            signet_address,
            "tb1pna0qxzm3z8vwre6h6vv5fh6m7pc5e9pk3nqtfxd77zad4jm8c25qs0s8hv".to_string()
        );
        assert_eq!(
            mainnet_address,
            "bc1pna0qxzm3z8vwre6h6vv5fh6m7pc5e9pk3nqtfxd77zad4jm8c25q88xgdr".to_string()
        );

        Ok(())
    }

    #[test]
    fn p2tr_to_spk_test() -> Result<(), String> {
        let signet_address = "tb1pna0qxzm3z8vwre6h6vv5fh6m7pc5e9pk3nqtfxd77zad4jm8c25qs0s8hv";
        let mainnet_address = "bc1pna0qxzm3z8vwre6h6vv5fh6m7pc5e9pk3nqtfxd77zad4jm8c25q88xgdr";

        let signet_spk_conversion = address_to_spk(Chain::Signet, signet_address).unwrap();
        let mainnet_spk_conversion = address_to_spk(Chain::Mainnet, mainnet_address).unwrap();

        // Appended with 0x5120 to make it a valid P2TR witness program.
        let expected_spk =
            hex::decode("51209f5e030b7111d8e1e757d31944df5bf0714c94368cc0b499bef0badacb67c2a8")
                .unwrap();

        assert_eq!(signet_spk_conversion, expected_spk);
        assert_eq!(mainnet_spk_conversion, expected_spk);

        Ok(())
    }

    #[test]
    fn encode_p2wsh_test() -> Result<(), String> {
        // OP_TRUE
        let witness_program: [u8; 32] =
            hex::decode("4ae81572f06e1b88fd5ced7a1a000945432e83e1551e6f721ee9c00b8cc33260")
                .unwrap()
                .try_into()
                .unwrap();

        let signet_address = encode_p2wsh(Chain::Signet, witness_program).unwrap();
        let mainnet_address = encode_p2wsh(Chain::Mainnet, witness_program).unwrap();

        assert_eq!(
            signet_address,
            "tb1qft5p2uhsdcdc3l2ua4ap5qqfg4pjaqlp250x7us7a8qqhrxrxfsqaqh7jw".to_string()
        );
        assert_eq!(
            mainnet_address,
            "bc1qft5p2uhsdcdc3l2ua4ap5qqfg4pjaqlp250x7us7a8qqhrxrxfsq2gp3gp".to_string()
        );

        Ok(())
    }

    #[test]
    fn p2wsh_to_spk_test() -> Result<(), String> {
        let signet_address = "tb1qft5p2uhsdcdc3l2ua4ap5qqfg4pjaqlp250x7us7a8qqhrxrxfsqaqh7jw";
        let mainnet_address = "bc1qft5p2uhsdcdc3l2ua4ap5qqfg4pjaqlp250x7us7a8qqhrxrxfsq2gp3gp";

        let signet_spk_conversion = address_to_spk(Chain::Signet, signet_address).unwrap();
        let mainnet_spk_conversion = address_to_spk(Chain::Mainnet, mainnet_address).unwrap();

        // Appended with 0x0020 to make it a valid P2WSH witness program.
        let expected_spk =
            hex::decode("00204ae81572f06e1b88fd5ced7a1a000945432e83e1551e6f721ee9c00b8cc33260")
                .unwrap();

        assert_eq!(signet_spk_conversion, expected_spk);
        assert_eq!(mainnet_spk_conversion, expected_spk);

        Ok(())
    }

    #[test]
    fn encode_p2wpkh_test() -> Result<(), String> {
        // Public key: 03f465315805ed271eb972e43d84d2a9e19494d10151d9f6adb32b8534bfd764ab

        let witness_program: [u8; 20] = hex::decode("841b80d2cc75f5345c482af96294d04fdd66b2b7")
            .unwrap()
            .try_into()
            .unwrap();

        let signet_address = encode_p2wpkh(Chain::Signet, witness_program).unwrap();
        let mainnet_address = encode_p2wpkh(Chain::Mainnet, witness_program).unwrap();

        assert_eq!(
            signet_address,
            "tb1qssdcp5kvwh6nghzg9tuk99xsflwkdv4hz2m8un".to_string()
        );
        assert_eq!(
            mainnet_address,
            "bc1qssdcp5kvwh6nghzg9tuk99xsflwkdv4hgvq58q".to_string()
        );

        Ok(())
    }

    #[test]
    fn p2wpkh_to_spk_test() -> Result<(), String> {
        let signet_address = "tb1qssdcp5kvwh6nghzg9tuk99xsflwkdv4hz2m8un";
        let mainnet_address = "bc1qssdcp5kvwh6nghzg9tuk99xsflwkdv4hgvq58q";

        let signet_spk_conversion = address_to_spk(Chain::Signet, signet_address).unwrap();
        let mainnet_spk_conversion = address_to_spk(Chain::Mainnet, mainnet_address).unwrap();

        // Appended with 0x0014 to make it a valid P2WPKH witness program.
        let expected_spk = hex::decode("0014841b80d2cc75f5345c482af96294d04fdd66b2b7").unwrap();

        assert_eq!(signet_spk_conversion, expected_spk);
        assert_eq!(mainnet_spk_conversion, expected_spk);

        Ok(())
    }
}
