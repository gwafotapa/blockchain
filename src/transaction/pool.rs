use std::collections::HashSet;
use std::fmt;
use std::iter::FromIterator;
use std::ops::Index;

use crate::transaction::Transaction;
// use crate::utxo::Utxo;

#[derive(fmt::Debug)]
pub struct TransactionPool {
    transactions: Vec<Transaction>,
}

impl TransactionPool {
    pub fn new() -> Self {
        Self {
            transactions: Vec::new(),
        }
    }

    // pub fn update(&mut self, utxo_pool: &UtxoPool) -> Option<Vec<u8>> {
    //     if let Some(transaction) = Transaction::find(PROBABILITY_NEW_TRANSACTION, utxo_pool) {
    //         self.transactions.push(transaction);
    //     }

    //     if self.transactions.len() < self.propagated + SEND {
    //         return None;
    //     }

    //     let mut bytes = Vec::new();
    //     bytes.push(b't'); // 't' stands for 'transaction'
    //     for transaction in &self.transactions[self.propagated..] {
    //         bytes.extend(transaction.serialize());
    //     }
    //     self.propagated = self.transactions.len();
    //     Some(bytes)
    // }

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

    // pub fn utxo(&self, input: TransactionInput) -> Option<Utxo> {
    //     input.utxo(self)
    // }
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
