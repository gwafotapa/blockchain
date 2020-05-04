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

    pub fn push(&mut self, block: Block) {
        assert!(block.height() > 0, "block height cannot be zero");
        let parent_hash = block.hash_prev_block();
        if self.chain.get(parent_hash).is_some() {
            if block.height() > self.height() {
                self.top_hash = block.hash();
            }
            self.push_orphans_of(&block);
            self.chain.insert(block.hash(), block);
        } else {
            self.orphans.insert(block.hash(), block);
        }
    }

    pub fn push_orphans_of(&mut self, parent: &Block) {
        let hashes: Vec<_> = self
            .orphans
            .iter()
            .filter(|(_, o)| *o.hash_prev_block() == parent.hash())
            .map(|(h, _)| h)
            .copied()
            .collect();
        for hash in hashes {
            let orphan = self.orphans.remove(&hash).unwrap();
            if orphan.height() > self.height() {
                self.top_hash = orphan.hash();
            }
            self.push_orphans_of(&orphan);
            self.chain.insert(hash, orphan);
        }
    }

    pub fn contains(&self, block: &Block) -> bool {
        self.chain.contains_key(&block.hash()) || self.orphans.contains_key(&block.hash())
    }

    pub fn is_longer_with(&self, block: &Block) -> bool {
        self.chain.contains_key(block.hash_prev_block()) && self.height() < block.height()
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
