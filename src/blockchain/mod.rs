use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt;

use self::error::BlockchainError;
use crate::block::Block;
use crate::common::Hash as BlockHash;
use crate::transaction::{Transaction, TransactionInput};
use crate::utxo::Utxo;

#[derive(Debug)]
pub struct Blockchain {
    chain: HashMap<BlockHash, Block>,
    top_hash: BlockHash,
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis = Block::genesis();
        let top_hash = genesis.hash();
        let mut chain = HashMap::new();
        chain.insert(top_hash, genesis);
        Self { chain, top_hash }
    }

    pub fn push(&mut self, block: Block) -> Result<(), BlockchainError> {
        if self.contains(&block) {
            return Err(BlockchainError::KnownBlock);
        }
        if self.parent(&block).is_none() {
            return Err(BlockchainError::OrphanBlock);
        }
        if block.height() > self.height() {
            self.top_hash = block.hash();
        }
        self.chain.insert(block.hash(), block);
        Ok(())
    }

    pub fn transaction_delta(&self, new_top: &Block) -> (Vec<Transaction>, Vec<Transaction>) {
        let old_top = self.top();
        let parent = self.common_parent(old_top, new_top).unwrap();
        let old_transactions = self.new_transactions_between(parent, old_top);
        let new_transactions = self.new_transactions_between(parent, new_top);
        (old_transactions, new_transactions)
    }

    fn new_transactions_between<'a>(
        &'a self,
        parent: &'a Block,
        mut child: &'a Block,
    ) -> Vec<Transaction> {
        let mut transactions = vec![];
        while child != parent {
            transactions.extend(child.transactions().clone());
            child = self.parent(child).unwrap();
        }
        transactions
    }

    pub fn contains(&self, block: &Block) -> bool {
        self.chain.contains_key(&block.hash())
    }

    pub fn parent(&self, block: &Block) -> Option<&Block> {
        self.chain.get(&block.hash_prev_block())
    }

    pub fn common_parent<'a>(
        &'a self,
        mut block1: &'a Block,
        mut block2: &'a Block,
    ) -> Option<&'a Block> {
        if block1.is_genesis() {
            return Some(block1);
        } else if block2.is_genesis() {
            return Some(block2);
        } else if self.parent(block1).is_none() || self.parent(block2).is_none() {
            return None;
        }
        while block1 != block2 {
            match block1.height().cmp(&block2.height()) {
                Ordering::Less => block2 = self.parent(block2).unwrap(),
                Ordering::Greater => block1 = self.parent(block1).unwrap(),
                Ordering::Equal => {
                    block1 = self.parent(block1).unwrap();
                    block2 = self.parent(block2).unwrap();
                }
            }
        }
        Some(block1)
    }

    // TODO: Should return an option
    pub fn get_utxo_from<'a>(&'a self, input: &TransactionInput, mut block: &'a Block) -> Utxo {
        loop {
            if let Some(utxo) = block.get_utxo_from(input) {
                return utxo;
            }
            block = self.parent(block).unwrap();
        }
    }

    pub fn top(&self) -> &Block {
        &self.chain[&self.top_hash]
    }

    pub fn height(&self) -> usize {
        self.top().height()
    }

    pub fn len(&self) -> usize {
        1 + self.height()
    }

    pub fn chain(&self) -> &HashMap<BlockHash, Block> {
        &self.chain
    }

    pub fn top_hash(&self) -> &BlockHash {
        &self.top_hash
    }
}

impl fmt::Display for Blockchain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Blockchain (blocks: {}, length: {}) {{",
            self.chain.len(),
            self.len()
        )?;
        let mut chain: Vec<_> = self.chain.iter().map(|(_, b)| b).collect();
        chain.sort_by_key(|b| b.height());
        for block in &chain {
            write!(
                f,
                "\n  height: {}\n  id: {}\n  parent: {}\n",
                block.height(),
                format!("{:#x}", block.id()),
                format!("{:#x}", block.hash_prev_block())
            )?;
        }
        write!(f, "}}\n")
    }
}

impl Eq for Blockchain {}

impl PartialEq for Blockchain {
    fn eq(&self, other: &Self) -> bool {
        let ch1: HashSet<BlockHash> = self.chain.iter().map(|(h, _)| h).copied().collect();
        let ch2: HashSet<BlockHash> = other.chain.iter().map(|(h, _)| h).copied().collect();
        if ch1.symmetric_difference(&ch2).next().is_some() {
            return false;
        }
        true
    }
}

pub mod error;
