use std::collections::HashSet;
use std::fmt;
use std::iter::FromIterator;
use std::ops::Index;

use crate::transaction::Transaction;

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

    pub fn transactions(&self) -> &[Transaction] {
        &self.transactions
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
