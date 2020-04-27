use rand::seq::IteratorRandom;
use rand::Rng;

use crate::common::{NODES, PROBABILITY_SPEND};
use crate::transaction::{Transaction, TransactionInput, TransactionOutput};
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

    pub fn remove(&mut self, input: &TransactionInput) {
        self.utxos
            .iter()
            .position(|utxo| *utxo.input() == *input)
            .and_then(|i| Some(self.utxos.remove(i)));
    }

    pub fn initiate(&mut self) -> Option<Transaction> {
        if self.utxos.is_empty() {
            return None;
        }
        let mut rng = rand::thread_rng();
        match rng.gen_bool(PROBABILITY_SPEND) {
            false => None,
            true => {
                let inputs_len = rng.gen_range(1, self.utxos.len() + 1);
                let mut inputs = Vec::with_capacity(inputs_len);
                let indices = (0..self.utxos.len()).choose_multiple(&mut rng, inputs_len);
                // indices.sort_by(|a, b| b.cmp(a));
                let mut amount = 0;
                // for index in indices {
                //     let input = self.utxos.remove(index);
                //     amount += input.amount();
                //     inputs.push(input);
                // }
                for index in indices {
                    let utxo = &self.utxos[index];
                    amount += utxo.amount();
                    inputs.push(*utxo.input());
                }
                let mut outputs = Vec::new();
                loop {
                    let amount1 = rng.gen_range(1, amount + 1);
                    // let mut recipient;
                    // loop {
                    //     recipient = rng.gen_range(0, NODES);
                    //     if recipient != self.id {
                    //         break;
                    //     }
                    // }
                    let recipient = rng.gen_range(0, NODES);
                    let output = TransactionOutput::new(amount1, recipient);
                    outputs.push(output);
                    amount -= amount1;
                    if amount == 0 {
                        break;
                    }
                }
                let transaction = Transaction::new(inputs, outputs);
                Some(transaction)
            }
        }
    }

    pub fn process(&mut self, transaction: &Transaction) {
        // let wallet_inputs = self.utxos.iter().map(|u| *u.input()).collect::<Vec<_>>();
        // for input in transaction.inputs() {
        //     if !wallet_inputs.contains(input) {
        //         return;
        //     }
        // }
        for input in transaction.inputs() {
            self.remove(input)
        }
        for (vout, &output) in transaction.outputs().iter().enumerate() {
            if output.puzzle() != self.id {
                continue;
            }
            let input = TransactionInput::new(transaction.id(), vout);
            let utxo = Utxo::new(input, output);
            self.add(utxo);
        }
        // if transaction.inputs()[0].puzzle() == self.id {
        //     for utxo in transaction.inputs() {
        //         if utxo.puzzle() != self.id {
        //             return Err(InvalidTransaction);
        //         }
        //         match self.utxos.iter().position(|u| *u == *utxo) {
        //             Some(index) => {
        //                 self.utxos.remove(index);
        //             }
        //             None => return Err(InvalidTransaction),
        //         }
        //     }
        //     Ok(())
        // } else {
        //     for utxo in transaction.outputs() {
        //         if utxo.puzzle() == self.id {
        //             self.add(*utxo)
        //         }
        //     }
        // Ok(())
        // }
    }
}
