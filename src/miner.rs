use rand::Rng;

use crate::block::Block;
use crate::common::MINE_NEW_BLOCK_PROBA;
use crate::transaction::TransactionPool;

pub struct Miner {
    block: Option<Block>,
}

impl Miner {
    pub fn new() -> Self {
        Self { block: None }
    }

    pub fn new_from(top: &Block, transaction_pool: &TransactionPool) -> Self {
        let mut miner = Self::new();
        miner.mine_from(top, transaction_pool);
        miner
    }

    pub fn mine_from(&mut self, top: &Block, transaction_pool: &TransactionPool) {
        if let Some(block) = self.block.as_ref() {
            if block.hash() == top.hash() {
                return;
            }
        }
        let transactions = transaction_pool.select();
        let block = Block::new(1 + top.height(), top.hash(), transactions);
        self.block = Some(block)
    }

    pub fn mine(&mut self) -> Option<Block> {
        let mut rng = rand::thread_rng();
        match rng.gen_bool(MINE_NEW_BLOCK_PROBA) {
            false => None,
            true => self.block.take(),
        }
    }
}
