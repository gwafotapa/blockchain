use sha2::{Digest, Sha256};
use std::convert::TryInto;
use std::fmt;
use std::iter;

use self::header::BlockHeader;
use crate::common::{Hash, GENESIS_BLOCK_HASH_PREV_BLOCK};
use crate::transaction::Transaction;

// const GENESIS_BLOCK_HASH_MERKLE_ROOT: &[u8; 32] =
//     &hex!("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b");

#[derive(Clone, Debug)]
pub struct Block {
    height: usize,
    header: BlockHeader,
    // transactions: Vec<Transaction>,
}

impl Block {
    pub fn genesis() -> Self {
        Self {
            height: 0,
            header: BlockHeader::new(
                Hash::from(GENESIS_BLOCK_HASH_PREV_BLOCK),
                // hash_merkle_root: *Hash::from_slice(GENESIS_BLOCK_HASH_MERKLE_ROOT),
            ),
            // transactions: Vec::new(),
        }
    }

    // TODO: use a single argument 'parent: &Block' instead or add another function ?
    pub fn new(height: usize, hash_prev_block: Hash) -> Self {
        // assert!(
        //     transactions.len().is_power_of_two(),
        //     "Number of transactions is not a power of 2"
        // );
        Self {
            height,
            header: BlockHeader::new(
                hash_prev_block, // hash_merkle_root: Transaction::hash_merkle_root(&transactions),
            ), // transactions,
        }
    }

    pub fn child(&self) -> Self {
        Self::new(1 + self.height(), self.hash())
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn hash_prev_block(&self) -> &Hash {
        &self.header.hash_prev_block()
    }

    //     pub fn hash_merkle_root(&self) -> Hash {
    //         self.header.hash_merkle_root
    //     }

    //     pub fn transactions(&self) -> &Vec<Transaction> {
    //         &self.transactions
    //     }

    pub fn hash(&self) -> Hash {
        let mut hasher = Sha256::new();
        hasher.input(self.header.serialize());
        let hash = hasher.result_reset();
        hasher.input(hash);
        hasher.result()
    }

    pub fn serialize(&self) -> Vec<u8> {
        iter::once(b'b')
            .chain(self.height.to_be_bytes().iter().copied())
            .chain(self.header.serialize())
            .collect()
        // let mut bytes = Vec::new();
        // bytes.extend(&self.height.to_be_bytes());
        // bytes.extend(self.header.serialize());
        // bytes.extend(self.hash_prev_block().as_slice());
        // bytes.extend(self.hash_merkle_root().as_slice());
        // for transaction in &self.transactions {
        //     bytes.extend(transaction.serialize());
        // }
        // bytes
    }

    pub fn deserialize<B>(bytes: B) -> Self
    where
        B: AsRef<[u8]>,
    {
        Self::from(bytes)
    }
}

impl<B> From<B> for Block
where
    B: AsRef<[u8]>,
{
    fn from(bytes: B) -> Self {
        let bytes = bytes.as_ref();
        let mut i = 1;
        let height = usize::from_be_bytes(bytes[i..i + 8].try_into().unwrap());
        i += 8;
        let header = BlockHeader::deserialize(&bytes[i..]);
        Self { height, header }
    }
}

pub mod header;

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
