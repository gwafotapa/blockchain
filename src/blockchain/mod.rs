use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

use self::error::BlockchainError;
use crate::block::Block;
use crate::utxo::{Utxo, UtxoId};
use crate::Hash as BlockHash;
use crate::Hash as TransactionId;

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
        if self.contains(block.id()) {
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

    /// Finds the two shortest lists of consecutive blocks joining two blocks
    ///
    /// Computes the closest common parent A of the two given blocks B and C, then returns:
    /// - the list of blocks between B (included) and A (not included) in decreasing height.
    /// - the list of blocks between C (included) and A (not included) in decreasing height.
    pub fn block_delta(&self, block1: &Block, block2: &Block) -> (Vec<Block>, Vec<Block>) {
        let parent = self.common_parent(block1, block2).unwrap();
        let list1 = self.range_of_blocks(block1, parent);
        let list2 = self.range_of_blocks(block2, parent);
        (list1, list2)
    }

    fn range_of_blocks<'a>(&'a self, mut child: &'a Block, parent: &'a Block) -> Vec<Block> {
        let mut blocks = VecDeque::new();
        while child != parent {
            blocks.push_front(child.clone());
            child = self.parent(child).unwrap();
        }
        Vec::from(blocks)
    }

    pub fn contains(&self, block_id: BlockHash) -> bool {
        // self.chain.contains_key(&block.hash())
        self.chain
            .iter()
            .any(|(_hash, block)| block.id() == block_id)
    }

    pub fn contains_tx(&self, txid: &TransactionId) -> bool {
        let mut block = self.top();
        loop {
            if block.contains(txid) {
                return true;
            }
            if block.is_genesis() {
                return false;
            }
            block = self.parent(block).unwrap();
        }
    }

    pub fn parent(&self, block: &Block) -> Option<&Block> {
        self.chain.get(block.hash_prev_block())
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

    pub fn get_utxo<'a>(&'a self, utxo_id: &UtxoId, mut block: &'a Block) -> Utxo {
        loop {
            if let Some(utxo) = block.get_utxo(utxo_id) {
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
                "\n  height: {}\n  id: {:x}\n  parent: {:x}\n",
                block.height(),
                block.id(),
                block.hash_prev_block()
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
