use std::sync::{mpsc::Sender, Arc};

use crate::transaction::Transaction;

const PROBABILITY_NEW_TRANSACTION: f64 = 1.0 / 1000000.0;
const SEND: usize = 1 << 2;

pub struct Ledger {
    transactions: Vec<Transaction>,
    propagated: usize,
}

impl Ledger {
    pub fn new() -> Self {
        Self {
            transactions: Vec::new(),
            propagated: 0,
        }
    }

    pub fn update(&mut self) -> Option<Vec<u8>> {
        if let Some(transaction) = Transaction::find(PROBABILITY_NEW_TRANSACTION) {
            let bytes = transaction.to_bytes();
            self.transactions.push(transaction);
            Some(bytes)
        } else {
            None
        }
    }

    pub fn send(&mut self, txs: &[(usize, Sender<Arc<Vec<u8>>>)]) {
        if self.transactions.len() < self.propagated + SEND {
            return;
        }
        let mut bytes = Vec::new();
        for transaction in &self.transactions[self.propagated..] {
            bytes.extend(transaction.to_bytes());
        }
        self.propagated = self.transactions.len();
        let bytes = Arc::new(bytes);
        for tx in txs {
            tx.1.send(Arc::clone(&bytes)).unwrap();
        }
    }

    pub fn add(&mut self, new: Vec<Transaction>) {
        self.transactions.extend(new);
    }

    pub fn next_batch(&self) -> Option<&[Transaction]> {
        if self.transactions.len() < self.propagated + SEND {
            None
        } else {
            Some(&self.transactions[self.propagated..])
        }
    }

    pub fn has_next_batch(&self) -> bool {
        self.next_batch().is_some()
    }

    // TODO: better algorithmic ? Use HashSet for difference ?
    // What about propagated transactions ?
    pub fn archive(&mut self, records: &Vec<Transaction>) {
        self.transactions.retain(|x| !records.contains(x));
    }
}
