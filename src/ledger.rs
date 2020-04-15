use std::sync::{mpsc::Sender, Arc};

use crate::transaction::Transaction;

const PROBABILITY_NEW_TRANSACTION: f64 = 1.0 / 1000000.0;
const SEND: usize = 1 << 6;

pub struct Ledger {
    transactions: Vec<Transaction>,
    sent: usize, // TODO: use 'next_batch' or 'next_batch_size' instead of 'sent'
}

impl Ledger {
    pub fn new() -> Self {
        Self {
            transactions: Vec::new(),
            sent: 0,
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
        if self.transactions.len() < self.sent + SEND {
            return;
        }
        let mut bytes = Vec::new();
        for transaction in &self.transactions[self.sent..] {
            bytes.extend(transaction.to_bytes());
        }
        self.sent = self.transactions.len();
        let bytes = Arc::new(bytes);
        for tx in txs {
            tx.1.send(Arc::clone(&bytes)).unwrap();
        }
    }

    pub fn add(&mut self, list: Vec<Transaction>) {
        self.transactions.extend(list);
    }

    pub fn next_batch(&self) -> Option<&[Transaction]> {
        if self.transactions.len() > self.sent + SEND {
            Some(&self.transactions[self.sent..])
        } else {
            None
        }
    }

    pub fn has_next_batch(&self) -> bool {
        self.next_batch().is_some()
    }
}
