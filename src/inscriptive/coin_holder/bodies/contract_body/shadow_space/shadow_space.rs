use sparse_merkle_tree::{
    blake2b::Blake2bHasher, default_store::DefaultStore, SparseMerkleTree, H256,
};
use std::collections::HashMap;

use crate::transmutative::hash::{Hash, HashTag};

/// Account key.
#[allow(non_camel_case_types)]
type ACCOUNT_KEY = [u8; 32];

/// Satoshi amount.
#[allow(non_camel_case_types)]
type SATOSHI_AMOUNT = u64;

/// Sati-satoshi amount.
#[allow(non_camel_case_types)]
type SATI_SATOSHI_AMOUNT = u128;

/// A struct for representing a shadow space of a contract.
pub struct ShadowSpace {
    // Total allocated BTC value of the entire shadow space.
    allocs_sum: SATOSHI_AMOUNT,

    // Allocated BTC values of each account.
    allocs: HashMap<ACCOUNT_KEY, SATI_SATOSHI_AMOUNT>,

    // Sparse tree of the allocated BTC values of each account.
    allocs_sparse_tree: SparseMerkleTree<Blake2bHasher, H256, DefaultStore<H256>>,
}

impl Clone for ShadowSpace {
    fn clone(&self) -> Self {
        // Reconstruct the sparse tree from the allocations
        let mut allocs_sparse_tree =
            SparseMerkleTree::<Blake2bHasher, H256, DefaultStore<H256>>::default();

        // Update the sparse tree with the allocated BTC values of each account.
        for (account_key, alloc_value) in self.allocs.iter() {
            let alloc_value_hash = alloc_value
                .to_le_bytes()
                .hash(Some(HashTag::ShadowAllocValue));

            // Update the sparse tree with the allocated BTC value of the account.
            let _ = allocs_sparse_tree.update(H256::from(*account_key), H256::from(alloc_value_hash));
        }

        Self {
            allocs_sum: self.allocs_sum,
            allocs: self.allocs.clone(),
            allocs_sparse_tree,
        }
    }
}

impl ShadowSpace {
    /// Constructs a fresh new shadow space.
    pub fn fresh_new() -> Self {
        Self {
            allocs_sum: 0,
            allocs: HashMap::new(),
            allocs_sparse_tree:
                SparseMerkleTree::<Blake2bHasher, H256, DefaultStore<H256>>::default(),
        }
    }
    /// Constructs a fresh new shadow space.
    pub fn new(
        allocs_sum: SATOSHI_AMOUNT,
        allocs: HashMap<ACCOUNT_KEY, SATI_SATOSHI_AMOUNT>,
    ) -> Self {
        // Construct the sparse tree of the allocated BTC values of each account.
        let mut allocs_sparse_tree =
            SparseMerkleTree::<Blake2bHasher, H256, DefaultStore<H256>>::default();

        // Update the sparse tree with the allocated BTC values of each account.
        for (account_key, alloc_value) in allocs.iter() {
            // Hash the allocated BTC value.
            let alloc_value_hash = H256::from(
                alloc_value
                    .to_le_bytes()
                    .hash(Some(HashTag::ShadowAllocValue)),
            );

            // Update the sparse tree with the allocated BTC value of the account.
            let _ = allocs_sparse_tree.update(H256::from(*account_key), alloc_value_hash);
        }

        // Return the shadow space.
        let shadow_space = Self {
            allocs_sum: allocs_sum,
            allocs: allocs,
            allocs_sparse_tree: allocs_sparse_tree,
        };

        // Return the shadow space.
        shadow_space
    }

    /// Returns the allocations sum.
    pub fn allocs_sum(&self) -> SATOSHI_AMOUNT {
        self.allocs_sum
    }

    /// Returns the number of allocations.
    pub fn allocs_len(&self) -> usize {
        self.allocs.len()
    }

    /// Returns a clone of the allocations map.
    pub fn allocs(&self) -> &HashMap<ACCOUNT_KEY, SATI_SATOSHI_AMOUNT> {
        &self.allocs
    }

    /// Inserts an allocation into the shadow space.
    pub fn insert_alloc(
        &mut self,
        account_key: ACCOUNT_KEY,
        alloc_value: SATI_SATOSHI_AMOUNT,
    ) -> bool {
        // Hash the allocated BTC value.
        let alloc_value_hash = alloc_value
            .to_le_bytes()
            .hash(Some(HashTag::ShadowAllocValue));

        // Update the allocations map.
        // Return false if the allocation was already present.
        if self.allocs.insert(account_key, alloc_value).is_some() {
            return false;
        }

        // Update the sparse tree.
        let _ =self.allocs_sparse_tree
            .update(H256::from(account_key), H256::from(alloc_value_hash));

        // Return true if the allocation was inserted.
        true
    }

    /// Removes an allocation from the shadow space.
    pub fn remove_alloc(&mut self, account_key: ACCOUNT_KEY) -> bool {
        // Remove the allocation from the allocations map.
        // Return false if the allocation was not found.
        if self.allocs.remove(&account_key).is_none() {
            return false;
        }

        // Remove the allocation from the sparse tree by updating with the default value.
        let _ = self.allocs_sparse_tree
            .update(H256::from(account_key), H256::default());

        // Return true if the allocation was removed.
        true
    }

    /// Updates the allocations sum.
    pub fn update_allocs_sum(&mut self, new_value: SATOSHI_AMOUNT) {
        // Update the allocations sum.
        self.allocs_sum = new_value;
    }

    /// Hashes the shadow space with the `ShadowSpace` tag.
    pub fn tagged_hash(&self) -> [u8; 32] {
        let mut preimage: Vec<u8> = Vec::<u8>::new();

        preimage.extend(self.allocs_sum.to_le_bytes());

        let sparse_tree_root = self.allocs_sparse_tree.root();

        preimage.extend(sparse_tree_root.as_slice());

        preimage.hash(Some(HashTag::ShadowSpace))
    }
}
