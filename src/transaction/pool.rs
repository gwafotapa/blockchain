use rand::seq::IteratorRandom;
use std::collections::HashSet;
use std::fmt;

use crate::block::Block;
use crate::constants::TXS_PER_BLOCK;
use crate::transaction::{Transaction, TransactionError};

#[derive(Debug)]
pub struct TransactionPool {
    transactions: HashSet<Transaction>,
}

impl TransactionPool {
    pub fn new() -> Self {
        Self {
            transactions: HashSet::new(),
        }
    }

    pub fn size(&self) -> usize {
        self.transactions.len()
    }

    pub fn add(&mut self, transaction: Transaction) -> Result<(), TransactionError> {
        if self.transactions.insert(transaction) {
            Ok(())
        } else {
            Err(TransactionError::PoolHasTransaction)
        }
    }

    pub fn remove(&mut self, transaction: &Transaction) -> Result<(), TransactionError> {
        if self.transactions.remove(transaction) {
            Ok(())
        } else {
            Err(TransactionError::UnknownTransaction)
        }
    }

    pub fn verify(&self, transaction: &Transaction) -> Result<(), TransactionError> {
        for pool_transaction in self.transactions() {
            for pool_input in pool_transaction.inputs() {
                for input in transaction.inputs() {
                    if input.utxo_id() == pool_input.utxo_id() {
                        return Err(TransactionError::PoolSpentUtxo(pool_transaction.id()));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn select(&self) -> Option<Vec<Transaction>> {
        if self.size() < TXS_PER_BLOCK {
            return None;
        }
        Some(
            self.transactions
                .iter()
                .choose_multiple(&mut rand::thread_rng(), TXS_PER_BLOCK)
                .iter()
                .map(|&tx| tx.clone())
                .collect(),
        )
    }

    /// Remove the block transactions from the pool
    ///
    /// When a fork occurs, valid received transactions may be deemed invalid if they concern
    /// the other chain. For this reason, there may be no transaction to remove from the pool
    /// when adopting the new chain in the event we lose the race. Hence the ok() call instead
    /// of unwrap().
    pub fn process(&mut self, block: &Block) {
        for transaction in block.transactions() {
            self.remove(transaction).ok();
        }
    }

    pub fn process_all(&mut self, blocks: &[Block]) {
        for block in blocks {
            self.process(block);
        }
    }

    pub fn undo(&mut self, block: &Block) {
        for transaction in block.transactions() {
            self.add(transaction.clone()).unwrap();
        }
    }

    pub fn undo_all(&mut self, blocks: &[Block]) {
        for block in blocks.iter().rev() {
            self.undo(block);
        }
    }

    pub fn transactions(&self) -> &HashSet<Transaction> {
        &self.transactions
    }
}

impl fmt::Display for TransactionPool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Transaction pool ({}) {{\n", self.size())?;
        for transaction in &self.transactions {
            write!(f, "  {}\n", format!("{:#x}", transaction.id()))?;
        }
        write!(f, "}}\n")
    }
}

impl Eq for TransactionPool {}

impl PartialEq for TransactionPool {
    fn eq(&self, other: &Self) -> bool {
        self.transactions
            .symmetric_difference(&other.transactions)
            .next()
            .is_none()
    }
}
