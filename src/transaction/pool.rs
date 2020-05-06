use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::fmt;
use std::iter::FromIterator;
use std::ops::Index;

use crate::common::TXS_PER_BLOCK;
use crate::transaction::Transaction;

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

    pub fn contains(&self, transaction: &Transaction) -> bool {
        self.transactions.contains(transaction)
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

    pub fn add_all(&mut self, transactions: Vec<Transaction>) {
        for transaction in transactions {
            self.add(transaction)
        }
    }

    pub fn remove_all(&mut self, transactions: &[Transaction]) {
        for transaction in transactions {
            self.remove(transaction);
        }
    }

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
