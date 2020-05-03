use std::collections::HashMap;

use crate::block::Block;
use crate::common::Hash;

type BlockHash = Hash;

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

    pub fn top(&self) -> &Block {
        &self.chain[&self.top_hash]
    }

    pub fn height(&self) -> usize {
        self.top().height()
    }

    pub fn len(&self) -> usize {
        1 + self.height()
    }
}
