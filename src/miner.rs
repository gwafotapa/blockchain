use rand::Rng;

use crate::block::Block;
use crate::constants::MINE_NEW_BLOCK_PROBA;
use crate::transaction::TransactionPool;

pub struct Miner {
    block: Option<Block>,
}

impl Miner {
    pub fn new() -> Self {
        Self { block: None }
    }

    pub fn mine(&mut self, top: &Block, transaction_pool: &TransactionPool) -> Option<Block> {
        self.mine_from(top, transaction_pool);
        if self.block.is_some() {
            let mut rng = rand::thread_rng();
            match rng.gen_bool(MINE_NEW_BLOCK_PROBA) {
                true => self.block.take(),
                false => None,
            }
        } else {
            None
        }
    }

    pub fn mine_from(&mut self, top: &Block, transaction_pool: &TransactionPool) {
        if let Some(block) = self.block.as_ref() {
            if block.hash_prev_block() == &top.hash() {
                return;
            }
        }
        self.block = transaction_pool
            .select()
            .map(|transactions| Block::new(top, transactions).unwrap());
    }

    pub fn discard_block(&mut self) {
        self.block = None;
    }
}
