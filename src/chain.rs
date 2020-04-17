use crate::{block::Block, transaction::Transaction};

pub struct Blockchain {
    chain: Vec<Block>,
    mined_block: Option<Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        Self {
            chain: vec![Block::genesis()],
            mined_block: None,
        }
    }

    pub fn push(&mut self, block: Block) {
        self.chain.push(block);
        self.mined_block = None;
    }

    pub fn len(&self) -> usize {
        self.chain.len()
    }

    pub fn last(&self) -> &Block {
        self.chain.last().unwrap()
    }

    pub fn mined_block(&self) -> Option<&Block> {
        self.mined_block.as_ref()
    }

    pub fn has_mined_block(&self) -> bool {
        self.mined_block.is_some()
    }

    pub fn take_mined_block(&mut self) -> Block {
        self.mined_block.take().unwrap()
    }

    pub fn set_mined_block(&mut self, transactions: Vec<Transaction>) {
        self.mined_block = Some(Block::new(
            self.last().height(),
            self.last().hash(),
            transactions,
        ));
    }
}
