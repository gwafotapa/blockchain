use hex::ToHex;
use sha2::{Digest, Sha256};
use std::convert::TryInto;
use std::fmt;
use std::iter;

use self::header::BlockHeader;
use crate::common::{Hash, GENESIS_BLOCK_HASH_PREV_BLOCK, HEADER_BYTES};
use crate::transaction::{Transaction, TransactionInput};
use crate::utxo::{Utxo, UtxoId};

pub use self::error::BlockError;

#[derive(Clone, Debug)]
pub struct Block {
    height: usize,
    header: BlockHeader,
    transactions: Vec<Transaction>,
}

impl Block {
    pub fn genesis() -> Self {
        let transactions = Vec::new();
        let header = BlockHeader::new(
            Hash::from(GENESIS_BLOCK_HASH_PREV_BLOCK),
            Transaction::hash_merkle_root(&transactions),
        );
        Self {
            height: 0,
            header,
            transactions,
        }
    }

    pub fn new(parent: &Block, transactions: Vec<Transaction>) -> Result<Self, BlockError> {
        if !transactions.len().is_power_of_two() {
            return Err(BlockError::WrongTransactionCount);
        }
        let header = BlockHeader::new(parent.hash(), Transaction::hash_merkle_root(&transactions));
        Ok(Self {
            height: 1 + parent.height(),
            header,
            transactions,
        })
    }

    pub fn get_utxo_from(&self, input: &TransactionInput) -> Option<Utxo> {
        for transaction in &self.transactions {
            if transaction.id() == input.txid() {
                let utxo_id = UtxoId::new(input.txid(), input.vout());
                let utxo_data = transaction.outputs()[input.vout()].0;
                let utxo = Utxo::new(utxo_id, utxo_data);
                return Some(utxo);
            }
        }
        None
    }

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
            .chain(self.transactions.len().to_be_bytes().iter().copied())
            .chain(self.header.serialize())
            .chain(self.transactions.iter().flat_map(|tx| tx.serialize()))
            .collect()
    }

    pub fn deserialize<B>(bytes: B) -> Self
    where
        B: AsRef<[u8]>,
    {
        Self::from(bytes)
    }

    pub fn is_genesis(&self) -> bool {
        self.height == 0
    }

    pub fn transaction_count(&self) -> usize {
        self.transactions.len()
    }

    pub fn id(&self) -> Hash {
        self.hash()
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn hash_prev_block(&self) -> Hash {
        self.header.hash_prev_block()
    }

    pub fn hash_merkle_root(&self) -> Hash {
        self.header.hash_merkle_root()
    }

    pub fn transactions(&self) -> &Vec<Transaction> {
        &self.transactions
    }
}

impl Eq for Block {}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.hash() == other.hash()
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
        let transactions_len = usize::from_be_bytes(bytes[i..i + 8].try_into().unwrap());
        i += 8;
        let header = BlockHeader::deserialize(&bytes[i..i + HEADER_BYTES]);
        i += HEADER_BYTES;
        let mut transactions = Vec::with_capacity(transactions_len);
        for _j in 0..transactions_len {
            let (transaction, size) = Transaction::deserialize(&bytes[i..]);
            transactions.push(transaction);
            i += size;
        }
        Self {
            height,
            header,
            transactions,
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Block {{\n  height: {}\n  hash_prev_block: {}\n  transactions: {}\n",
            self.height,
            format!("{:#x}", self.hash_prev_block()),
            self.transaction_count()
        )?;
        for transaction in &self.transactions {
            write!(f, "    ")?;
            transaction.id().write_hex(f)?;
            write!(f, "\n")?;
        }
        write!(f, "}}\n")
    }
}

pub mod error;
pub mod header;
