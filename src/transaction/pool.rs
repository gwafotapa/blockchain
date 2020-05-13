use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::fmt;
use std::iter::FromIterator;
use std::ops::Index;

use crate::block::Block;
use crate::constants::TXS_PER_BLOCK;
use crate::transaction::{Transaction, TransactionError};

// TODO: Should it be a vector or a hashmap ?
#[derive(Debug)]
pub struct TransactionPool {
    transactions: Vec<Transaction>,
}

impl TransactionPool {
    pub fn new() -> Self {
        Self {
            transactions: Vec::new(),
        }
    }

    pub fn size(&self) -> usize {
        self.transactions.len()
    }

    // TODO: use push instead of add ?
    pub fn add(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    pub fn remove(&mut self, transaction: &Transaction) -> Option<Transaction> {
        self.position(transaction)
            .map(|i| self.transactions.remove(i))
    }

    // pub fn contains(&self, transaction: &Transaction) -> bool {
    //     self.transactions.contains(transaction)
    // }

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

    pub fn position(&self, transaction: &Transaction) -> Option<usize> {
        self.transactions.iter().position(|tx| tx == transaction)
    }

    pub fn select(&self) -> Option<Vec<Transaction>> {
        if self.size() < TXS_PER_BLOCK {
            return None;
        }
        Some(
            self.transactions
                .choose_multiple(&mut rand::thread_rng(), TXS_PER_BLOCK)
                .cloned()
                .collect(),
        )
    }

    pub fn process(&mut self, block: &Block) {
        for transaction in block.transactions() {
            self.remove(transaction);
        }
    }

    pub fn process_all(&mut self, blocks: &[Block]) {
        for block in blocks {
            self.process(block);
        }
    }

    pub fn undo(&mut self, block: &Block) {
        for transaction in block.transactions() {
            self.add(transaction.clone());
        }
    }

    pub fn undo_all(&mut self, blocks: &[Block]) {
        for block in blocks.iter().rev() {
            self.undo(block);
        }
    }

    // pub fn add_all(&mut self, transactions: Vec<Transaction>) {
    //     for transaction in transactions {
    //         self.add(transaction)
    //     }
    // }

    // pub fn remove_all(&mut self, transactions: &[Transaction]) {
    //     for transaction in transactions {
    //         self.remove(transaction);
    //     }
    // }

    pub fn transactions(&self) -> &Vec<Transaction> {
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
        let p1 = HashSet::<Transaction>::from_iter(self.transactions().iter().cloned());
        let p2 = HashSet::<Transaction>::from_iter(other.transactions().iter().cloned());
        p1.symmetric_difference(&p2).next().is_none()
    }
}

impl Index<usize> for TransactionPool {
    type Output = Transaction;

    fn index(&self, index: usize) -> &Self::Output {
        &self.transactions[index]
    }
}
