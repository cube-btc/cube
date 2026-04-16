use crate::transmutative::codec::prefix::Prefix;
use crate::transmutative::hash::{Hash, HashTag};
use secp::{MaybePoint, Point, Scalar};
use std::cmp::Ordering;
use std::vec;

const LEAF_VERSION: u8 = 0xc0;
const POINT_WITH_UNKNOWN_DISCRETE_LOGARITHM: [u8; 33] = [
    0x02, 0x50, 0x92, 0x9b, 0x74, 0xc1, 0xa0, 0x49, 0x54, 0xb7, 0x8b, 0x4b, 0x60, 0x35, 0xe9, 0x7a,
    0x5e, 0x07, 0x8a, 0x5a, 0x0f, 0x28, 0xec, 0x96, 0xd5, 0x47, 0xbf, 0xee, 0x9a, 0xce, 0x80, 0x3a,
    0xc0,
];

pub trait P2TR {
    fn taproot(&self) -> Option<TapRoot>;
}

#[derive(Clone)]
pub enum Branch {
    Leaf(TapLeaf),
    Branch(Box<TapBranch>),
}

#[derive(Clone)]
pub struct TapLeaf {
    leaf_version: u8,
    tap_script: Vec<u8>,
}

impl TapLeaf {
    pub fn new(tap_script: Vec<u8>) -> TapLeaf {
        TapLeaf {
            leaf_version: LEAF_VERSION,
            tap_script,
        }
    }

    pub fn new_version(tap_script: Vec<u8>, leaf_version: u8) -> TapLeaf {
        TapLeaf {
            leaf_version,
            tap_script,
        }
    }

    pub fn tapleaf_hash(&self) -> [u8; 32] {
        hash_tap_leaf(&self.tap_script, self.leaf_version)
    }

    pub fn into_branch(&self) -> Branch {
        Branch::Leaf(self.clone())
    }

    pub fn tap_script(&self) -> Vec<u8> {
        self.tap_script.clone()
    }
}

#[derive(Clone)]
pub struct TapBranch {
    left_branch: Branch,
    right_branch: Branch,
}

impl TapBranch {
    pub fn new(first: Branch, second: Branch) -> TapBranch {
        let first_branch = match &first {
            Branch::Leaf(leaf) => leaf.tapleaf_hash(),
            Branch::Branch(branch) => branch.tapbranch_hash(),
        };

        let second_branch = match &second {
            Branch::Leaf(leaf) => leaf.tapleaf_hash(),
            Branch::Branch(branch) => branch.tapbranch_hash(),
        };

        match &first_branch.cmp(&second_branch) {
            Ordering::Less => TapBranch {
                left_branch: first,
                right_branch: second,
            },
            _ => TapBranch {
                left_branch: second,
                right_branch: first,
            },
        }
    }

    pub fn tapbranch_hash(&self) -> [u8; 32] {
        let left_branch = match &self.left_branch {
            Branch::Branch(branch) => branch.tapbranch_hash(),
            Branch::Leaf(leaf) => leaf.tapleaf_hash(),
        };

        let right_branch = match &self.right_branch {
            Branch::Branch(branch) => branch.tapbranch_hash(),
            Branch::Leaf(leaf) => leaf.tapleaf_hash(),
        };

        hash_tap_branch(left_branch, right_branch)
    }

    pub fn into_branch(&self) -> Branch {
        Branch::Branch(Box::new(self.clone()))
    }
}

#[derive(Clone)]
pub struct TapRoot {
    inner_key: Point,
    tree: Option<TapTree>,
}

impl TapRoot {
    pub fn key_and_script_path_single(inner_key: Point, leaf: TapLeaf) -> TapRoot {
        TapRoot {
            inner_key,
            tree: Some(TapTree::new(vec![leaf])),
        }
    }

    pub fn key_and_script_path_multi(inner_key: Point, leaves: Vec<TapLeaf>) -> TapRoot {
        TapRoot {
            inner_key,
            tree: Some(TapTree::new(leaves)),
        }
    }

    pub fn key_path_only(inner_key: Point) -> TapRoot {
        TapRoot {
            inner_key,
            tree: None,
        }
    }

    pub fn script_path_only_single(leaf: TapLeaf) -> TapRoot {
        let inner_key = Point::from_slice(&POINT_WITH_UNKNOWN_DISCRETE_LOGARITHM).unwrap();
        TapRoot {
            inner_key,
            tree: Some(TapTree::new(vec![leaf])),
        }
    }

    pub fn script_path_only_multi(leaves: Vec<TapLeaf>) -> TapRoot {
        let inner_key = Point::from_slice(&POINT_WITH_UNKNOWN_DISCRETE_LOGARITHM).unwrap();
        TapRoot {
            inner_key,
            tree: Some(TapTree::new(leaves)),
        }
    }

    pub fn inner_key(&self) -> Point {
        self.inner_key
    }

    pub fn inner_key_parity(&self) -> bool {
        self.inner_key.parity().into()
    }

    pub fn inner_key_lifted(&self) -> Point {
        self.inner_key.negate_if(self.inner_key.parity())
    }

    pub fn tap_branch(&self) -> [u8; 32] {
        let uppermost_branch = match &self.tree {
            Some(tree) => tree.tap_branch(),
            None => [0x00u8; 32],
        };
        uppermost_branch
    }

    pub fn tap_tweak(&self) -> [u8; 32] {
        let inner_key_bytes = self.inner_key.serialize_xonly();
        let tap_branch_bytes = self.tap_branch();

        hash_tap_tweak(inner_key_bytes, tap_branch_bytes)
    }

    pub fn tweaked_key(&self) -> Option<Point> {
        if let Some(_) = &self.tree {
            let tweak = Scalar::from_slice(&self.tap_tweak()).ok()?;
            let tweaked_key = self.inner_key_lifted() + tweak.base_point_mul();

            match tweaked_key {
                MaybePoint::Valid(point) => Some(point),
                MaybePoint::Infinity => None,
            }
        } else {
            Some(self.inner_key_lifted())
        }
    }

    pub fn tweaked_key_parity(&self) -> Option<bool> {
        let tweaked_key = self.tweaked_key()?;
        Some(tweaked_key.parity().into())
    }

    pub fn spk(&self) -> Option<Vec<u8>> {
        let mut spk = vec![0x51, 0x20];
        let tweaked_key = self.tweaked_key()?;
        spk.extend(tweaked_key.serialize_xonly());
        Some(spk)
    }

    pub fn control_block(&self, index: usize) -> Option<ControlBlock> {
        let path = match &self.tree {
            Some(tree) => tree.path(index),
            None => return None,
        };

        let inner_key = self.inner_key();
        let parity: bool = self.tweaked_key_parity()?;

        Some(ControlBlock::new(inner_key, parity, path))
    }
    pub fn tree(&self) -> Option<TapTree> {
        self.tree.clone()
    }
}

#[derive(Clone)]
pub struct TapTree {
    leaves: Vec<TapLeaf>,
    tap_branch: [u8; 32],
}

impl TapTree {
    pub fn new(leaves: Vec<TapLeaf>) -> TapTree {
        let uppermost_branch = tree_builder(&leaves, None).0;

        let tap_branch = match &uppermost_branch {
            Branch::Leaf(leaf) => leaf.tapleaf_hash(),
            Branch::Branch(branch) => branch.tapbranch_hash(),
        };

        TapTree {
            leaves: leaves.clone(),
            tap_branch,
        }
    }

    pub fn leaves(&self) -> Vec<TapLeaf> {
        self.leaves.clone()
    }

    pub fn tap_branch(&self) -> [u8; 32] {
        self.tap_branch
    }

    pub fn path(&self, index: usize) -> Vec<u8> {
        // Given leaf index return the merkle path

        let path_vec = match tree_builder(&self.leaves, Some(index)).1 {
            Some(vec) => vec,
            None => panic!(),
        };
        path_vec
    }
}

// tree_builder returns given a vector of leaves, the tree root,
// and optionally a merkle path corresponding to some leaf
pub fn tree_builder(leaves: &Vec<TapLeaf>, index: Option<usize>) -> (Branch, Option<Vec<u8>>) {
    // Initialize path as empty
    let path: Vec<u8> = Vec::<u8>::new();

    match leaves.len() {
        0 => panic!("TapTree must be initialized with at least one TapLeaf."),
        1 => {
            // If single-leaf
            let branch: Branch = leaves[0].into_branch();
            match &index {
                Some(_) => (branch, Some(path)),
                None => (branch, None),
            }
        }
        _ => {
            let mut path: Vec<u8> = Vec::<u8>::new();
            let mut lookup: Option<Branch> = match index.clone() {
                Some(index) => Some(leaves[index].into_branch()),
                None => None,
            };

            // Number of TapTree levels is = log2(number of TapLeaves)
            let num_levels: u8 = (leaves.len() as f64).log2() as u8;

            let mut current_level: Vec<Branch> = Vec::new();
            let mut above_level: Vec<Branch> = Vec::new();

            // For each level of the TapTree
            for level in 0..(num_levels + 1) {
                // If it is the level zero, initialize current_level  with individual TapLeaves
                if level == 0 {
                    for i in 0..leaves.len() {
                        current_level.push(leaves[i].clone().into_branch());
                    }
                } else {
                    // If it is the level one or above, move above_level items into current_level, and reset above_level
                    current_level.clear();
                    current_level.extend(above_level.clone());
                    above_level.clear();
                }

                let mut iterator: usize = 0;
                let iterator_bound: usize = current_level.len();
                let operations: usize = match iterator_bound {
                    0 => panic!("This should not be the case."),
                    1 => 1,
                    _ => (iterator_bound / 2) + (iterator_bound % 2),
                };

                for _ in 0..operations {
                    match iterator_bound - iterator {
                        0 => panic!("This should not be the case."),
                        // last
                        1 => {
                            above_level.push(current_level[iterator].clone());
                            iterator += 1;
                        }
                        // two or more left in the current scope
                        _ => {
                            let first: Branch = current_level[iterator].clone();
                            let second: Branch = current_level[iterator + 1].clone();

                            let first_bytes = match &first {
                                Branch::Leaf(leaf) => leaf.tapleaf_hash(),
                                Branch::Branch(branch) => branch.tapbranch_hash(),
                            };

                            let second_bytes = match &second {
                                Branch::Leaf(leaf) => leaf.tapleaf_hash(),
                                Branch::Branch(branch) => branch.tapbranch_hash(),
                            };

                            let lookup_bytes = match &lookup {
                                Some(branch) => match branch {
                                    Branch::Leaf(leaf) => leaf.tapleaf_hash(),
                                    Branch::Branch(branch) => branch.tapbranch_hash(),
                                },
                                None => [0u8; 32],
                            };

                            // Lookup match?
                            let mut match_bool: bool = false;
                            if &first_bytes == &lookup_bytes {
                                path.extend(&second_bytes);
                                match_bool = true;
                            } else if &second_bytes == &lookup_bytes {
                                path.extend(&first_bytes);
                                match_bool = true;
                            }

                            let new_branch: TapBranch = TapBranch::new(first, second);

                            if match_bool {
                                lookup = Some(new_branch.into_branch());
                            }

                            above_level.push(new_branch.into_branch());
                            iterator += 2;
                        }
                    }
                }

                // At the end of each level, the iterator must have covered all branches of that level
                assert_eq!(iterator, iterator_bound);
            }

            // At the end, only the uppermost branch must be left
            assert_eq!(above_level.len(), 1);

            let branch: Branch = above_level[0].clone();

            match &index {
                Some(_) => (branch, Some(path)),
                None => (branch, None),
            }
        }
    }
}

pub struct ControlBlock {
    inner_key: Point,
    parity: bool,
    leaf_version: u8,
    path: Vec<u8>,
}

impl ControlBlock {
    pub fn new(inner_key: Point, parity: bool, path: Vec<u8>) -> ControlBlock {
        ControlBlock {
            inner_key,
            parity,
            leaf_version: LEAF_VERSION,
            path,
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::<u8>::new();

        match self.parity {
            false => vec.push(self.leaf_version),    // even parity
            true => vec.push(self.leaf_version + 1), // odd parity
        };

        vec.extend(self.inner_key.serialize_xonly().to_vec());
        vec.extend(self.path.clone());
        vec
    }
}

pub fn hash_tap_leaf(raw_script_bytes: &Vec<u8>, version: u8) -> [u8; 32] {
    let mut data: Vec<u8> = Vec::new();

    data.extend(&[version]);
    data.extend(raw_script_bytes.prefix_compact_size());

    data.hash(Some(HashTag::TapLeaf))
}

pub fn hash_tap_branch(left_branch_bytes: [u8; 32], right_branch_bytes: [u8; 32]) -> [u8; 32] {
    let mut data: Vec<u8> = Vec::new();

    data.extend(left_branch_bytes);
    data.extend(right_branch_bytes);

    data.hash(Some(HashTag::TapBranch))
}

pub fn hash_tap_tweak(inner_key_bytes: [u8; 32], tweak_bytes: [u8; 32]) -> [u8; 32] {
    let mut data: Vec<u8> = Vec::new();

    data.extend(inner_key_bytes);
    data.extend(tweak_bytes);

    data.hash(Some(HashTag::TapTweak))
}
