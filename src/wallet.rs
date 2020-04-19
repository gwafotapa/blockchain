use rand::Rng;

use crate::common::{NODES, PROBABILITY_SPEND};
use crate::transaction::Transaction;
use crate::utxo::Utxo;

pub struct Wallet {
    utxos: Vec<Utxo>,
}

impl Wallet {
    pub fn new(utxos: Vec<Utxo>) -> Self {
        Self { utxos }
    }

    pub fn manage(&mut self) -> Option<Transaction> {
        if self.utxos.is_empty() {
            return None;
        }
        let mut rng = rand::thread_rng();
        match rng.gen_bool(PROBABILITY_SPEND) {
            false => None,
            true => {
                let index = rng.gen_range(0, self.utxos.len());
                let input = self.utxos.remove(index);
                let mut recipient;
                loop {
                    recipient = rng.gen_range(0, NODES);
                    if recipient != input.puzzle() {
                        break;
                    }
                }
                let output = Utxo::new(input.amount(), recipient);
                let transaction = Transaction::new(input, output);
                Some(transaction)
            }
        }
    }
}
