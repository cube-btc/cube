#[cfg(test)]
mod taproot_tests {
    use cube::constructive::taproot::{ControlBlock, TapBranch, TapLeaf, TapRoot, TapTree};
    use secp::Point;
    use std::error::Error;

    #[test]
    fn test_tap_branch() -> Result<(), Box<dyn Error>> {
        // Test - Branch two TapLeaves

        let tap_leaf_1: TapLeaf = TapLeaf::new(vec![0xde, 0xad]);
        let tap_leaf_2: TapLeaf = TapLeaf::new(vec![0xbe, 0xef]);

        let tap_branch: TapBranch =
            TapBranch::new(tap_leaf_1.into_branch(), tap_leaf_2.into_branch());

        let expected: Vec<u8> =
            hex::decode("b220872a5f6915e7779e659c2925b4b6cef6c1792f2e7bed0ba6331631fa7c63")?;

        assert_eq!(tap_branch.tapbranch_hash().to_vec(), expected);

        // Test - Reversed order does not affect the branch

        let tap_branch_reversed: TapBranch =
            TapBranch::new(tap_leaf_2.into_branch(), tap_leaf_1.into_branch());

        assert_eq!(tap_branch_reversed.tapbranch_hash().to_vec(), expected);

        // Test - Branch two TapBranches

        let tap_leaf_3: TapLeaf = TapLeaf::new(vec![0xaa, 0xbb]);
        let tap_leaf_4: TapLeaf = TapLeaf::new(vec![0xcc, 0xdd]);

        let tap_branch_2: TapBranch =
            TapBranch::new(tap_leaf_3.into_branch(), tap_leaf_4.into_branch());

        let upper_tap_branch: TapBranch =
            TapBranch::new(tap_branch.into_branch(), tap_branch_2.into_branch());

        let expected_upper: Vec<u8> =
            hex::decode("a590e5a5cc3576cacb587676397bb8c7fa8645279ce740e5bf48bc7c25b1d813")?;

        assert_eq!(upper_tap_branch.tapbranch_hash().to_vec(), expected_upper);

        Ok(())
    }

    #[test]
    fn test_taproot_key_and_script_path() -> Result<(), Box<dyn Error>> {
        let tap_leaf: TapLeaf = TapLeaf::new(vec![0xaa, 0xbb, 0xcc]);

        // Test - with even inner key

        let inner_key_even = Point::from_slice(
            &hex::decode("028c17db0c798574086299e5041ffbcfa06bd501eb0e50914731bfbd2f3c9f980e")
                .unwrap(),
        )
        .unwrap();

        let taproot = TapRoot::key_and_script_path_single(inner_key_even, tap_leaf.clone());

        let expected_with_odd: Vec<u8> =
            hex::decode("51202e1a63521f2d72ff54da28cf8e114c6e3ce3ef497e9a6ac71b3e28e06446a218")?;

        assert_eq!(taproot.spk().unwrap(), expected_with_odd);

        // Test - with odd inner key

        let inner_key_odd = Point::from_slice(
            &hex::decode("037b55a1c853b28c398141c8fdf4eb69469430f019983af4be4b5aa7512936f295")
                .unwrap(),
        )
        .unwrap();

        let taproot = TapRoot::key_and_script_path_single(inner_key_odd, tap_leaf.clone());

        let expected_with_even: Vec<u8> =
            hex::decode("51208cda55510b8f99ec248ed9772e6a71537eb26142d6624d38426a7a1311b488e6")?;

        assert_eq!(taproot.spk().unwrap(), expected_with_even);

        Ok(())
    }

    #[test]
    fn test_taproot_key_path_only() -> Result<(), Box<dyn Error>> {
        // Test with even inner key

        let inner_key_even = Point::from_slice(
            &hex::decode("02d14c281713f15b608cc75d94717bbb1c2a4ff11e169c757f87a149daf61d54f0")
                .unwrap(),
        )
        .unwrap();

        let taproot_with_even_inner = TapRoot::key_path_only(inner_key_even);

        let expected_spk_with_inner =
            hex::decode("5120d14c281713f15b608cc75d94717bbb1c2a4ff11e169c757f87a149daf61d54f0")?;

        assert_eq!(
            taproot_with_even_inner.spk().unwrap(),
            expected_spk_with_inner
        );

        // Test with odd inner key

        let inner_key_odd = Point::from_slice(
            &hex::decode("03a2314467943d47cf102477b985d21c5ffa6512961b08906724f13e779cfed299")
                .unwrap(),
        )
        .unwrap();

        let taproot_with_odd_inner = TapRoot::key_path_only(inner_key_odd);

        let expected_spk_with_inner =
            hex::decode("5120a2314467943d47cf102477b985d21c5ffa6512961b08906724f13e779cfed299")?;

        assert_eq!(
            taproot_with_odd_inner.spk().unwrap(),
            expected_spk_with_inner
        );

        Ok(())
    }

    #[test]
    fn test_taproot_script_path_only() -> Result<(), Box<dyn Error>> {
        // Test with odd tweaked key

        let tap_leaf_with_odd = TapLeaf::new(vec![0x01, 0x23, 0xab, 0xcd]);
        let tap_root_with_odd = TapRoot::script_path_only_single(tap_leaf_with_odd.clone());

        let expected_spk =
            hex::decode("512085dbf94f892274c41acb75d48daf338c739d1157c70963912db526c4cad30d1a")?;
        assert_eq!(tap_root_with_odd.spk().unwrap(), expected_spk);
        assert_eq!(tap_root_with_odd.tweaked_key_parity().unwrap(), true);

        // Test with even tweaked key

        let tap_leaf_with_even = TapLeaf::new(vec![0x01, 0x23, 0xab, 0xcd, 0xef, 0xff]);
        let tap_root_with_even = TapRoot::script_path_only_single(tap_leaf_with_even.clone());

        let expected_spk =
            hex::decode("51201fbb64a309f43ee6a442cd293a9df3ce3bbb0864a2215a1091c06521021f9de4")?;
        assert_eq!(tap_root_with_even.spk().unwrap(), expected_spk);
        assert_eq!(tap_root_with_even.tweaked_key_parity().unwrap(), false);

        Ok(())
    }

    #[test]
    fn test_control_block() -> Result<(), Box<dyn Error>> {
        let tap_leaf_single: TapLeaf = TapLeaf::new(vec![0xaa, 0xbb, 0xcc]);
        let tap_root_single_leaf: TapRoot = TapRoot::script_path_only_multi(vec![tap_leaf_single]);

        let expected_cb: Vec<u8> =
            hex::decode("c050929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0")?;

        assert_eq!(
            tap_root_single_leaf.control_block(0).unwrap().to_vec(),
            expected_cb
        );

        let tap_leaf_1: TapLeaf = TapLeaf::new(vec![0xaa]);
        let tap_leaf_2: TapLeaf = TapLeaf::new(vec![0xbb]);
        let tap_leaf_3: TapLeaf = TapLeaf::new(vec![0xcc]);

        let leaves: Vec<TapLeaf> = vec![tap_leaf_1, tap_leaf_2, tap_leaf_3];

        let tap_root: TapRoot = TapRoot::script_path_only_multi(leaves);

        let expected_cb_1 =
            hex::decode("c050929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac091af7e676faf0d787dd6628f8d068756dd2de2473b94e5aa63915f168764e821fe06075904b2d09b06d544283b5ed7948355e691785c7b3e1a952a1a705151fe")?;

        let expected_cb_2 =
            hex::decode("c050929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0083b809ebc8a6e8077a1521d2621ef988887817d95691059b63db4efa6b354c8fe06075904b2d09b06d544283b5ed7948355e691785c7b3e1a952a1a705151fe")?;

        let expected_cb_3 =
            hex::decode("c050929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0823a89de31a35a726355b97b88e0f8fa0692fbf38630ebed328478f17c054a8c")?;

        assert_eq!(tap_root.control_block(0).unwrap().to_vec(), expected_cb_1);
        assert_eq!(tap_root.control_block(1).unwrap().to_vec(), expected_cb_2);
        assert_eq!(tap_root.control_block(2).unwrap().to_vec(), expected_cb_3);

        Ok(())
    }

    #[test]
    fn test_control_block_create() -> Result<(), Box<dyn Error>> {
        let inner_key = Point::from_slice(
            &hex::decode("03a2314467943d47cf102477b985d21c5ffa6512961b08906724f13e779cfed299")
                .unwrap(),
        )
        .unwrap();

        let path: Vec<u8> =
            hex::decode("0576e0a5d1c8fd852ab17ffac14e336b3143298fad1d3d9a302212ec9b1f8202")?;

        let control_block = ControlBlock::new(inner_key, true, path);

        let expected_cb = hex::decode("c1a2314467943d47cf102477b985d21c5ffa6512961b08906724f13e779cfed2990576e0a5d1c8fd852ab17ffac14e336b3143298fad1d3d9a302212ec9b1f8202")?;

        assert_eq!(control_block.to_vec(), expected_cb);

        Ok(())
    }

    #[test]
    fn test_tap_tree() -> Result<(), Box<dyn Error>> {
        let tap_leaf_1 = TapLeaf::new(vec![0xaa]);
        let tap_leaf_2 = TapLeaf::new(vec![0xbb]);
        let tap_leaf_3 = TapLeaf::new(vec![0xcc]);
        let tap_leaf_4 = TapLeaf::new(vec![0xdd]);
        let tap_leaf_5 = TapLeaf::new(vec![0xee]);
        let tap_leaf_6 = TapLeaf::new(vec![0xff]);
        let tap_leaf_7 = TapLeaf::new(vec![0x00]);
        let tap_leaf_8 = TapLeaf::new(vec![0x11]);
        let tap_leaf_9 = TapLeaf::new(vec![0x22]);
        let tap_leaf_10 = TapLeaf::new(vec![0x33]);
        let tap_leaf_11 = TapLeaf::new(vec![0x44]);
        let tap_leaf_12 = TapLeaf::new(vec![0x55]);

        let mut leaves = vec![];

        // Test single-leaf - aa
        leaves.push(tap_leaf_1.clone());
        let tap_tree = TapTree::new(leaves.clone());
        let expected =
            hex::decode("083b809ebc8a6e8077a1521d2621ef988887817d95691059b63db4efa6b354c8")?;
        assert_eq!(tap_tree.tap_branch().to_vec(), expected);

        // Test 2 leaves - aa bb
        leaves.push(tap_leaf_2.clone());
        let tap_tree = TapTree::new(leaves.clone());
        let expected =
            hex::decode("823a89de31a35a726355b97b88e0f8fa0692fbf38630ebed328478f17c054a8c")?;
        assert_eq!(tap_tree.tap_branch().to_vec(), expected);

        // Test 3 leaves - aa bb cc
        leaves.push(tap_leaf_3.clone());
        let tap_tree = TapTree::new(leaves.clone());
        let expected =
            hex::decode("fbacf98dc7eed29334d7f70ad70b78d8d0fd3362537f1f23d27fdbe7df302636")?;
        assert_eq!(tap_tree.tap_branch().to_vec(), expected);

        // Test 4 leaves - aa bb cc dd
        leaves.push(tap_leaf_4.clone());
        let tap_tree = TapTree::new(leaves.clone());
        let expected =
            hex::decode("4ab024178a74f8e2435cc88b8fd5c03cbb75d0e14b4e72e8388062b67be8e842")?;
        assert_eq!(tap_tree.tap_branch().to_vec(), expected);

        // Test 5 leaves - aa bb cc dd ee
        leaves.push(tap_leaf_5.clone());
        let tap_tree = TapTree::new(leaves.clone());
        let expected =
            hex::decode("fda09a939d87da777a274e0ad4232769445f15acd6b6e9d72053e4268354782d")?;
        assert_eq!(tap_tree.tap_branch().to_vec(), expected);

        // Test 6 leaves - aa bb cc dd ee ff
        leaves.push(tap_leaf_6.clone());
        let tap_tree = TapTree::new(leaves.clone());
        let expected =
            hex::decode("4ad803081bbcd04f49c4682d999ee748bf8400629a424f0c3dbad2638af45cc9")?;
        assert_eq!(tap_tree.tap_branch().to_vec(), expected);

        // Test 7 leaves - aa bb cc dd ee ff 00
        leaves.push(tap_leaf_7.clone());
        let tap_tree = TapTree::new(leaves.clone());
        let expected =
            hex::decode("73e54d9b7301cd6d8b528c16b801edba35347fcbf99da51abcc9727d43401ea7")?;
        assert_eq!(tap_tree.tap_branch().to_vec(), expected);

        // Test 8 leaves - aa bb cc dd ee ff 00 11
        leaves.push(tap_leaf_8.clone());
        let tap_tree = TapTree::new(leaves.clone());
        let expected =
            hex::decode("7648d42aead620a6ed02d82cc44a8e18a08da8ca1467928220ecf43ab308f195")?;
        assert_eq!(tap_tree.tap_branch().to_vec(), expected);

        // Test 9 leaves - aa bb cc dd ee ff 00 11 22
        leaves.push(tap_leaf_9.clone());
        let tap_tree = TapTree::new(leaves.clone());
        let expected =
            hex::decode("01efb5b091f906f27aa04dcb5a7a74938f736538a75df778acd66f3a968a310a")?;
        assert_eq!(tap_tree.tap_branch().to_vec(), expected);

        // Test 10 leaves - aa bb cc dd ee ff 00 11 22 33
        leaves.push(tap_leaf_10.clone());
        let tap_tree = TapTree::new(leaves.clone());
        let expected =
            hex::decode("3dad9105423be9dce1422e4f4f3ea6e49196104df08db7bcd8fd6d39591e79d4")?;
        assert_eq!(tap_tree.tap_branch().to_vec(), expected);

        // Test 11 leaves - aa bb cc dd ee ff 00 11 22 33 44
        leaves.push(tap_leaf_11.clone());
        let tap_tree = TapTree::new(leaves.clone());
        let expected =
            hex::decode("fe90a52c636872a7c7f1bc8faf59da361ad7d51d5bf88c883cc2dd268fa26b47")?;
        assert_eq!(tap_tree.tap_branch().to_vec(), expected);

        // Test 12 leaves - aa bb cc dd ee ff 00 11 22 33 44 55
        leaves.push(tap_leaf_12.clone());
        let tap_tree = TapTree::new(leaves.clone());
        let expected =
            hex::decode("44446fb50fce9c698734e1bfd10ed894baaed244dc7ce67e4bf12b1d38760c30")?;
        assert_eq!(tap_tree.tap_branch().to_vec(), expected);

        Ok(())
    }

    #[test]
    fn test_tap_tree_path() -> Result<(), Box<dyn Error>> {
        let tap_leaf_1: TapLeaf = TapLeaf::new(vec![0xaa]);
        let tap_leaf_2: TapLeaf = TapLeaf::new(vec![0xbb]);
        let tap_leaf_3: TapLeaf = TapLeaf::new(vec![0xcc]);
        let tap_leaf_4: TapLeaf = TapLeaf::new(vec![0xdd]);
        let tap_leaf_5: TapLeaf = TapLeaf::new(vec![0xee]);

        let mut leaves: Vec<TapLeaf> = vec![];

        // Test single-leaf - aa
        leaves.push(tap_leaf_1);
        leaves.push(tap_leaf_2);
        leaves.push(tap_leaf_3);
        leaves.push(tap_leaf_4);
        leaves.push(tap_leaf_5);

        let tap_tree: TapTree = TapTree::new(leaves.clone());

        let expected_path_1 =
            hex::decode("91af7e676faf0d787dd6628f8d068756dd2de2473b94e5aa63915f168764e8217f7b1fecf4af01c485881138c8484c4c7e6f537e896686a5e46d90e9b0c83692f6f920dc9dcb98ba04cff112f583969b4fa240bedf781759d6b6e0f2e74eb7fa")?;

        let expected_path_2 =
            hex::decode("083b809ebc8a6e8077a1521d2621ef988887817d95691059b63db4efa6b354c87f7b1fecf4af01c485881138c8484c4c7e6f537e896686a5e46d90e9b0c83692f6f920dc9dcb98ba04cff112f583969b4fa240bedf781759d6b6e0f2e74eb7fa")?;

        let expected_path_3 =
            hex::decode("b6537362191d9a5e0aa3a730b93b6f98a99ef63ed893bef4b9dfa7e3451eaf36823a89de31a35a726355b97b88e0f8fa0692fbf38630ebed328478f17c054a8cf6f920dc9dcb98ba04cff112f583969b4fa240bedf781759d6b6e0f2e74eb7fa")?;

        let expected_path_4 =
            hex::decode("fe06075904b2d09b06d544283b5ed7948355e691785c7b3e1a952a1a705151fe823a89de31a35a726355b97b88e0f8fa0692fbf38630ebed328478f17c054a8cf6f920dc9dcb98ba04cff112f583969b4fa240bedf781759d6b6e0f2e74eb7fa")?;

        let expected_path_5: Vec<u8> =
            hex::decode("4ab024178a74f8e2435cc88b8fd5c03cbb75d0e14b4e72e8388062b67be8e842")?;

        assert_eq!(tap_tree.path(0), expected_path_1);
        assert_eq!(tap_tree.path(1), expected_path_2);
        assert_eq!(tap_tree.path(2), expected_path_3);
        assert_eq!(tap_tree.path(3), expected_path_4);
        assert_eq!(tap_tree.path(4), expected_path_5);

        Ok(())
    }

    #[test]
    fn test_tap_tree_64() -> Result<(), Box<dyn Error>> {
        let mut leaves = Vec::<TapLeaf>::new();

        for i in 0..64 {
            leaves.push(TapLeaf::new(vec![i as u8]));
        }
        let tap_root = TapRoot::script_path_only_multi(leaves);

        let expected_spk =
            hex::decode("5120b88bb9de3afa63f0cd5b533f70a58f60004b65b6a1b6683a1ba766e37b11455b")?;
        assert_eq!(tap_root.spk().unwrap(), expected_spk);

        let expected_cb_leaf_0 = hex::decode("c150929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac01aac25408a4d28233cd325faefade9ef0fae76fcb1e35d08140045bbaa381b30c01491a3c808832bdf3bab5ea3208726c6eae12c3db3f8098919e145caa981de8fd6e8cdf1c53b7b1509958f4288d46fcc6c172dc9d32a52c0f8af4d5f86efc369632feaaca2e76395ae30e30fa5211fc0c099997a7de3a80d6ac566bdef300b7a41ea55777781977241267979150a1654dd92eecd7eb820b4aae57967a28952a2489c6a8c3011b12b89148d2abafa042d7982533826d3b911851abb34e7e741")?;
        assert_eq!(
            tap_root.control_block(0).unwrap().to_vec(),
            expected_cb_leaf_0
        );

        let expected_cb_leaf_10 = hex::decode("c150929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac033f0e118ce1dc6ee36199de7766e39d5a37da363e63cdb5342fac9e437c98261e2100cc38af83fde32fdb302ce68109844fa99f71ba58721f7cdbf7c3083ccae9602fd5610cc4ce5ff81afb18acd5140c2c2525e61e0ae7bfc335d1457df2352c33b3b99fa0737f5da94cfb3fe918e3b8467ed9d546588a117531672f48928657a41ea55777781977241267979150a1654dd92eecd7eb820b4aae57967a28952a2489c6a8c3011b12b89148d2abafa042d7982533826d3b911851abb34e7e741")?;
        assert_eq!(
            tap_root.control_block(10).unwrap().to_vec(),
            expected_cb_leaf_10
        );

        let expected_cb_leaf_45 = hex::decode("c150929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac03fe216e39e56269ca739d5e48e09bc93de208b9ebdbe524f665d28e103c86fe6c4154feadf35d5527875e82839e0878e9b6f18c4096c652830beee93cff38219923ebb8ebff4c5a8907da345ac47ce386249f745e8f2e942de33050358d20b289430f4b106bf5617e6d11333464d368b33b0433bf1f3d32ce840ecb65ac92d84c350786781aec83736c548e62ac04427a1747036cb212292bc4011aecb275e6326f6c6d5df019644b4aa8fa2116fe6c09bcc83bdedb621e2443a69218954063b")?;
        assert_eq!(
            tap_root.control_block(45).unwrap().to_vec(),
            expected_cb_leaf_45
        );

        let expected_cb_leaf_61 = hex::decode("c150929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac06925381e2d092124c53f87297a6e68f07ed3132a9761684bcaa475ea4fcf248dc870c97467df9cb37e9b481d0296b2660b23ef76ed7f84dee951c0d90b54aef97403758af8698bc5cdf75ca317b1036d1c0a33d9834962095693fc6b72ed68b2082edeb867fd98827cca5c1a0c7b517910712bb20e7c97d7ea50b273c5b19ddcaffb3ddcecd12cc515b9487bdd4b9497a9efa05e22b3c00bad374b7dce8c5f9c26f6c6d5df019644b4aa8fa2116fe6c09bcc83bdedb621e2443a69218954063b")?;
        assert_eq!(
            tap_root.control_block(61).unwrap().to_vec(),
            expected_cb_leaf_61
        );

        let expected_cb_leaf_63 = hex::decode("c150929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0a0a62d83f5b0ca6f2623bf7e2c347a9c8c4f950918cafad4ab742ab5c2ae04bebbc8d5c38114a3884f07c076229288ba618fe866ed324ab2e67b482f3c1965607403758af8698bc5cdf75ca317b1036d1c0a33d9834962095693fc6b72ed68b2082edeb867fd98827cca5c1a0c7b517910712bb20e7c97d7ea50b273c5b19ddcaffb3ddcecd12cc515b9487bdd4b9497a9efa05e22b3c00bad374b7dce8c5f9c26f6c6d5df019644b4aa8fa2116fe6c09bcc83bdedb621e2443a69218954063b")?;
        assert_eq!(
            tap_root.control_block(63).unwrap().to_vec(),
            expected_cb_leaf_63
        );

        Ok(())
    }
}
