use generic_array::{typenum::U32, GenericArray};
use hex_literal::hex;
use sha2::{Digest, Sha256};

use crate::transaction::Transaction;

type Hash = GenericArray<u8, U32>;

const GENESIS_BLOCK_HASH_PREV_BLOCK: &[u8; 32] =
    &hex!("0000000000000000000000000000000000000000000000000000000000000000");
const GENESIS_BLOCK_HASH_MERKLE_ROOT: &[u8; 32] =
    &hex!("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b");

pub struct Block {
    height: usize,
    header: BlockHeader,
    transactions: Vec<Transaction>,
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
            transactions: Vec::new(),
        }
    }

    pub fn new(height: usize, hash_prev_block: Hash, transactions: Vec<Transaction>) -> Self {
        assert!(
            transactions.len().is_power_of_two(),
            "Number of transactions is not a power of 2"
        );
        Self {
            height,
            header: BlockHeader {
                hash_prev_block,
                hash_merkle_root: Transaction::hash_merkle_root(&transactions),
            },
            transactions,
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

    pub fn transactions(&self) -> &Vec<Transaction> {
        &self.transactions
    }

    pub fn hash(&self) -> Hash {
        let mut hasher = Sha256::new();
        // hasher.input(self.to_bytes());
        hasher.input(self.serialize());
        hasher.result()
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        // bytes.extend(b"b"); // 'b' stands for 'block'
        bytes.extend(&self.height.to_be_bytes());
        bytes.extend(self.hash_prev_block().as_slice());
        bytes.extend(self.hash_merkle_root().as_slice());
        for transaction in &self.transactions {
            bytes.extend(transaction.serialize());
        }
        bytes
    }
}
