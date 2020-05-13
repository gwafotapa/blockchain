use merkle_cbt::merkle_tree::Merge;
use sha2::{Digest, Sha256};

use crate::Hash;

pub struct MergeHash {}

impl Merge for MergeHash {
    type Item = Hash;

    fn merge(left: &Self::Item, right: &Self::Item) -> Self::Item {
        let mut concat = Vec::from(left.as_slice());
        concat.extend_from_slice(right.as_slice());
        let mut hasher = Sha256::new();
        hasher.input(concat);
        hasher.result()
    }
}
