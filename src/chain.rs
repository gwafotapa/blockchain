use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;

use crate::block::Block;
use crate::common::Hash as BlockHash;

#[derive(Debug)]
pub struct Blockchain {
    chain: HashMap<BlockHash, Block>,
    orphans: HashMap<BlockHash, Block>,
    top_hash: BlockHash,
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis = Block::genesis();
        let top_hash = genesis.hash();
        let mut chain = HashMap::new();
        chain.insert(top_hash, genesis);
        Self {
            chain,
            orphans: HashMap::new(),
            top_hash,
        }
    }

    pub fn push(&mut self, block: Block) -> (Vec<Block>, Vec<Block>) {
        let old_top_hash = self.add(block);
        self.transaction_delta(old_top_hash)
    }

    pub fn add(&mut self, block: Block) -> BlockHash {
        assert!(block.height() > 0, "block height cannot be zero");
        let old_top_hash = self.top_hash;
        if self.chain.get(block.hash_prev_block()).is_some() {
            let block_hash = block.hash();
            if block.height() == 1 + self.height() {
                self.top_hash = block_hash;
            }
            self.chain.insert(block_hash, block);
            let orphans = self.remove_orphans_of(block_hash);
            for orphan in orphans {
                self.add(orphan);
            }
        } else {
            self.orphans.insert(block.hash(), block);
        }
        old_top_hash
    }

    pub fn remove_orphans_of(&mut self, block_hash: BlockHash) -> Vec<Block> {
        let hashes: Vec<_> = self
            .orphans
            .iter()
            .filter(|(_, o)| *o.hash_prev_block() == block_hash)
            .map(|(h, _)| h)
            .copied()
            .collect();
        hashes
            .iter()
            .map(|h| self.orphans.remove(h).unwrap())
            .collect()
    }

    pub fn transaction_delta(&self, old_top_hash: BlockHash) -> (Vec<Block>, Vec<Block>) {
        let old_top = self.chain.get(&old_top_hash).unwrap();
        let new_top = self.top();
        let parent = self.common_parent(old_top, new_top).unwrap();
        // TODO: Add a function for the next 6 lines
        let mut old_blocks = vec![];
        let mut old_block = old_top;
        while old_block != parent {
            old_blocks.push(old_block.clone());
            old_block = self.parent(old_block).unwrap();
        }
        let mut new_blocks = vec![];
        let mut new_block = new_top;
        while new_block != parent {
            new_blocks.push(new_block.clone());
            new_block = self.parent(new_block).unwrap();
        }
        (old_blocks, new_blocks)
    }

    pub fn contains(&self, block: &Block) -> bool {
        self.chain.contains_key(&block.hash()) || self.orphans.contains_key(&block.hash())
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

    pub fn top(&self) -> &Block {
        &self.chain[&self.top_hash]
    }

    pub fn height(&self) -> usize {
        self.top().height()
    }

    pub fn len(&self) -> usize {
        1 + self.height()
    }

    pub fn top_hash(&self) -> &BlockHash {
        &self.top_hash
    }
}

impl fmt::Display for Blockchain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "  chain:\n")?;
        for (_, block) in &self.chain {
            write!(f, "    {}\n", block)?;
        }
        write!(f, "  orphans:\n")?;
        for (_, block) in &self.orphans {
            write!(f, "    {}\n", block)?;
        }
        Ok(())
    }
}
