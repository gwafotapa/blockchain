use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

use crate::block::Block;
use crate::constants::UTXO_HASH_INIT;
use crate::error::blockchain::BlockchainError;
use crate::transaction::Transaction;
use crate::utxo::{Utxo, UtxoData, UtxoId};
use crate::Hash as BlockHash;
use crate::Hash as TransactionId;

#[derive(Debug)]
pub struct Blockchain {
    chain: HashMap<BlockHash, Block>,
    top_hash: BlockHash,
    initial_utxos: HashMap<UtxoId, UtxoData>,
}

impl Blockchain {
    pub fn new(initial_utxos: HashMap<UtxoId, UtxoData>) -> Self {
        let genesis = Block::genesis();
        let top_hash = genesis.hash();
        let mut chain = HashMap::new();
        chain.insert(top_hash, genesis);
        Self {
            chain,
            top_hash,
            initial_utxos,
        }
    }

    pub fn push(&mut self, block: Block) -> Result<(), BlockchainError> {
        if self.contains(block.id()) {
            return Err(BlockchainError::KnownBlock);
        }
        if self.get_parent_of(&block).is_none() {
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
    pub fn path(&self, block1: &Block, block2: &Block) -> (Vec<Block>, Vec<Block>) {
        let parent = self.common_parent(block1, block2).unwrap();
        let list1 = self.range_of_blocks(block1, parent);
        let list2 = self.range_of_blocks(block2, parent);
        (list1, list2)
    }

    fn range_of_blocks<'a>(&'a self, mut child: &'a Block, parent: &'a Block) -> Vec<Block> {
        let mut blocks = VecDeque::new();
        while child != parent {
            blocks.push_front(child.clone());
            child = self.get_parent_of(child).unwrap();
        }
        Vec::from(blocks)
    }

    pub fn contains(&self, block_id: BlockHash) -> bool {
        self.chain
            .iter()
            .any(|(_hash, block)| block.id() == block_id)
    }

    pub fn check_id_of(&self, block: &Block) -> Result<(), BlockchainError> {
        if self.contains(block.id()) {
            Err(BlockchainError::KnownBlock)
        } else {
            Ok(())
        }
    }

    pub fn contains_tx(
        &self,
        txid: &TransactionId,
        start: Option<&Block>,
        end: Option<&Block>,
    ) -> bool {
        let start = start.unwrap_or_else(|| self.genesis());
        let mut block = end.unwrap_or_else(|| self.top());
        loop {
            if block.contains(txid) {
                return true;
            }
            if block == start {
                return false;
            }
            block = self.get_parent_of(block).unwrap();
        }
    }

    pub fn check_txid_of(&self, transaction: &Transaction) -> Result<(), BlockchainError> {
        if self.contains_tx(transaction.id(), None, None) {
            Err(BlockchainError::KnownTransactionId)
        } else {
            Ok(())
        }
    }

    pub fn get_parent_of(&self, block: &Block) -> Option<&Block> {
        self.chain.get(block.hash_prev_block())
    }

    pub fn parent_of(&self, block: &Block) -> Result<&Block, BlockchainError> {
        self.get_parent_of(block)
            .ok_or(BlockchainError::OrphanBlock)
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
        } else if self.get_parent_of(block1).is_none() || self.get_parent_of(block2).is_none() {
            return None;
        }

        while block1 != block2 {
            match block1.height().cmp(&block2.height()) {
                Ordering::Less => block2 = self.get_parent_of(block2).unwrap(),
                Ordering::Greater => block1 = self.get_parent_of(block1).unwrap(),
                Ordering::Equal => {
                    block1 = self.get_parent_of(block1).unwrap();
                    block2 = self.get_parent_of(block2).unwrap();
                }
            }
        }
        Some(block1)
    }

    pub fn get_utxo<'a>(&'a self, utxo_id: &UtxoId, mut block: &'a Block) -> Utxo {
        if *utxo_id.txid() == TransactionId::from(UTXO_HASH_INIT) {
            let utxo_data = self.initial_utxos[utxo_id];
            return Utxo::new(*utxo_id, utxo_data);
        }
        loop {
            if let Some(utxo) = block.get_utxo(utxo_id) {
                return utxo;
            }
            block = self.get_parent_of(block).unwrap();
        }
    }

    pub fn check_txids_of(&self, block: &Block) -> Result<(), BlockchainError> {
        for transaction in block.transactions() {
            if self.contains_tx(transaction.id(), None, self.get_parent_of(block)) {
                return Err(BlockchainError::KnownTransactionId);
            }
        }
        Ok(())
    }

    pub fn top(&self) -> &Block {
        &self.chain[&self.top_hash]
    }

    pub fn genesis(&self) -> &Block {
        self.chain
            .iter()
            .filter_map(|(_, b)| if b.is_genesis() { Some(b) } else { None })
            .next()
            .unwrap()
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

    pub fn initial_utxos(&self) -> &HashMap<UtxoId, UtxoData> {
        &self.initial_utxos
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
