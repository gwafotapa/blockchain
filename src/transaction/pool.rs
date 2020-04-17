use crate::transaction::Transaction;

const PROBABILITY_NEW_TRANSACTION: f64 = 1.0 / 1000000.0;
const SEND: usize = 1 << 2;

pub struct TransactionPool {
    transactions: Vec<Transaction>,
    propagated: usize,
}

impl TransactionPool {
    pub fn new() -> Self {
        Self {
            transactions: Vec::new(),
            propagated: 0,
        }
    }

    pub fn update(&mut self) -> Option<Vec<u8>> {
        if let Some(transaction) = Transaction::find(PROBABILITY_NEW_TRANSACTION) {
            self.transactions.push(transaction);
        }

        if self.transactions.len() < self.propagated + SEND {
            return None;
        }

        let mut bytes = Vec::new();
        bytes.push(b't'); // 't' stands for 'transaction'
        for transaction in &self.transactions[self.propagated..] {
            bytes.extend(transaction.to_bytes());
        }
        self.propagated = self.transactions.len();
        Some(bytes)
    }

    pub fn add(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
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
    pub fn remove(&mut self, records: &Vec<Transaction>) {
        self.transactions.retain(|x| !records.contains(x));
    }
}
