use secp256k1::PublicKey;

use crate::transaction::{Transaction, TransactionInput};
use crate::utxo::Utxo;

pub struct Wallet {
    public_key: PublicKey,
    utxos: Vec<Utxo>,
}

impl Wallet {
    pub fn new(public_key: PublicKey, utxos: Vec<Utxo>) -> Self {
        Self { public_key, utxos }
    }

    pub fn add(&mut self, utxo: Utxo) {
        if utxo.public_key() != self.public_key() {
            panic!("Invalid public key")
        }
        self.utxos.push(utxo);
    }

    pub fn remove(&mut self, input: &TransactionInput) {
        self.utxos
            .iter()
            .position(|utxo| *utxo.input() == *input)
            .and_then(|i| Some(self.utxos.remove(i)));
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
            if output.public_key() != self.public_key() {
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

    pub fn public_key(&self) -> PublicKey {
        self.public_key
    }

    pub fn utxos(&self) -> &[Utxo] {
        &self.utxos
    }
}
