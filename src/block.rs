use generic_array::{typenum::U32, GenericArray};
use hex_literal::hex;
use sha2::{Digest, Sha256};

// use crate::transaction::Transactions;

type Hash = GenericArray<u8, U32>;

const GENESIS_BLOCK_HASH_PREV_BLOCK: &[u8; 32] =
    &hex!("0000000000000000000000000000000000000000000000000000000000000000");
const GENESIS_BLOCK_HASH_MERKLE_ROOT: &[u8; 32] =
    &hex!("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b");

pub struct Block {
    height: usize,
    header: BlockHeader,
    // transactions: Transactions,
}

pub struct BlockHeader {
    hash_prev_block: Hash,
    hash_merkle_root: Hash,
    // nonce: u32,
}

impl Block {
    pub fn genesis() -> Self {
        Self {
            height: 0,
            header: BlockHeader {
                hash_prev_block: *Hash::from_slice(GENESIS_BLOCK_HASH_PREV_BLOCK),
                hash_merkle_root: *Hash::from_slice(GENESIS_BLOCK_HASH_MERKLE_ROOT),
            },
        }
    }

    pub fn new(height: usize, hash_prev_block: Hash, hash_merkle_root: Hash) -> Self {
        Self {
            height,
            header: BlockHeader {
                hash_prev_block,
                hash_merkle_root,
            },
        }
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn hash_prev_block(&self) -> Hash {
        self.header.hash_prev_block
    }

    pub fn hash_merkle_root(&self) -> Hash {
        self.header.hash_merkle_root
    }

    pub fn hash(&self) -> Hash {
        let mut hasher = Sha256::new();
        hasher.input(self.to_bytes());
        hasher.result()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.height.to_be_bytes());
        bytes.extend_from_slice(self.hash_prev_block().as_slice());
        bytes.extend_from_slice(self.hash_merkle_root().as_slice());
        bytes
    }
}
