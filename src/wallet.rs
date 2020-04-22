use rand::Rng;
use std::result;

use crate::common::{NODES, PROBABILITY_SPEND};
use crate::transaction::{InvalidTransaction, Transaction};
use crate::utxo::Utxo;

pub struct Wallet {
    id: usize,
    utxos: Vec<Utxo>,
}

impl Wallet {
    pub fn new(id: usize, utxos: Vec<Utxo>) -> Self {
        Self { id, utxos }
    }

    pub fn add(&mut self, utxo: Utxo) {
        if utxo.puzzle() != self.id {
            panic!("Invalid puzzle")
        }
        self.utxos.push(utxo);
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
                let mut amount = input.amount();
                let mut outputs = Vec::new();
                loop {
                    let amountp = rng.gen_range(1, amount + 1);
                    let mut recipient;
                    loop {
                        recipient = rng.gen_range(0, NODES);
                        if recipient != input.puzzle() {
                            break;
                        }
                    }
                    let output = Utxo::new(amountp, recipient);
                    outputs.push(output);
                    amount -= amountp;
                    if amount == 0 {
                        break;
                    }
                }
                let transaction = Transaction::new(input, outputs);
                Some(transaction)
            }
        }
    }

    pub fn process(&mut self, transaction: &Transaction) -> result::Result<(), InvalidTransaction> {
        if transaction.input().puzzle() == self.id {
            match self
                .utxos
                .iter()
                .position(|utxo| *utxo == transaction.input())
            {
                Some(index) => {
                    self.utxos.remove(index);
                    Ok(())
                }
                None => Err(InvalidTransaction),
            }
        } else {
            for utxo in transaction.outputs() {
                if utxo.puzzle() == self.id {
                    self.add(*utxo)
                }
            }
            Ok(())
        }
    }
}
