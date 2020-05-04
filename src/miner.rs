use rand::Rng;

use crate::block::Block;
use crate::common::MINE_NEW_BLOCK_PROBA;

pub struct Miner {
    mined: Option<Block>,
}

impl Miner {
    pub fn new(top: &Block) -> Self {
        Self {
            mined: Some(top.child(vec![])),
        }
    }

    pub fn mine(&mut self) -> Option<Block> {
        let mut rng = rand::thread_rng();
        match rng.gen_bool(MINE_NEW_BLOCK_PROBA) {
            false => None,
            true => {
                let block = self.mined.take().unwrap();
                self.mined = Some(block.child(vec![]));
                Some(block)
            }
        }
    }
}
