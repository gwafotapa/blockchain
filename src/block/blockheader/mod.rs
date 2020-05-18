use std::convert::TryInto;

use self::target::Target;
use crate::constants::TARGET;
use crate::Hash;

#[derive(Clone, Debug)]
pub struct BlockHeader {
    hash_prev_block: Hash,
    hash_merkle_root: Hash,
    target: Target,
    nonce: u32,
}

impl BlockHeader {
    pub fn new(hash_prev_block: Hash, hash_merkle_root: Hash) -> Self {
        Self {
            hash_prev_block,
            hash_merkle_root,
            target: TARGET.into(),
            nonce: 0,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.hash_prev_block
            .into_iter()
            .chain(self.hash_merkle_root.into_iter())
            .chain(self.target.serialize())
            .chain(self.nonce.to_be_bytes().to_vec())
            .collect()
    }

    pub fn deserialize<B>(bytes: B) -> Self
    where
        B: AsRef<[u8]>,
    {
        Self::from(bytes)
    }

    pub fn inc_nonce(&mut self) {
        self.nonce += 1;
    }

    pub fn hash_prev_block(&self) -> &Hash {
        &self.hash_prev_block
    }

    pub fn hash_merkle_root(&self) -> &Hash {
        &self.hash_merkle_root
    }

    pub fn target(&self) -> Target {
        self.target
    }
}

impl<B> From<B> for BlockHeader
where
    B: AsRef<[u8]>,
{
    fn from(bytes: B) -> Self {
        let bytes = bytes.as_ref();
        let mut i = 0;
        let hash_prev_block = *Hash::from_slice(&bytes[i..i + 32]);
        i += 32;
        let hash_merkle_root = *Hash::from_slice(&bytes[i..i + 32]);
        i += 32;
        let target = Target::deserialize(&bytes[i..i + 4]);
        i += 4;
        let nonce = u32::from_be_bytes(bytes[i..i + 4].try_into().unwrap());
        Self {
            hash_prev_block,
            hash_merkle_root,
            target,
            nonce,
        }
    }
}

pub mod target;
