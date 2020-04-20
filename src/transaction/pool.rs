use crate::transaction::Transaction;

// const PROBABILITY_NEW_TRANSACTION: f64 = 1.0 / 1000000.0;
// const SEND: usize = 1 << 2;

pub struct TransactionPool {
    transactions: Vec<Transaction>,
    // propagated: usize,
}

impl TransactionPool {
    pub fn new() -> Self {
        Self {
            transactions: Vec::new(),
            // propagated: 0,
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

    pub fn transactions(&self) -> &[Transaction] {
        &self.transactions
    }

    // pub fn propagated(&self) -> usize {
    //     self.propagated
    // }

    pub fn size(&self) -> usize {
        self.transactions.len()
    }

    // pub fn set_propagated(&mut self, propagated: usize) {
    //     self.propagated = propagated;
    // }

    pub fn add(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    pub fn contains(&self, transaction: Transaction) -> bool {
        self.transactions.contains(&transaction)
    }

    // pub fn next_batch(&self) -> Option<&[Transaction]> {
    //     if self.transactions.len() < self.propagated + SEND {
    //         None
    //     } else {
    //         Some(&self.transactions[self.propagated..])
    //     }
    // }

    // pub fn has_next_batch(&self) -> bool {
    //     self.next_batch().is_some()
    // }

    // TODO: better algorithmic ? Use HashSet for difference ?
    // What about propagated transactions ?
    // pub fn remove(&mut self, records: &Vec<Transaction>) {
    //     self.transactions.retain(|x| !records.contains(x));
    // }
}
