use secp256k1::PublicKey;

use crate::block::Block;
use crate::transaction::{Transaction, TransactionInput};
use crate::utxo::{Utxo, UtxoData, UtxoId};

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
            .position(|utxo| utxo.txid() == input.txid() && utxo.vout() == input.vout())
            .and_then(|i| Some(self.utxos.remove(i)));
    }

    pub fn process(&mut self, transaction: &Transaction) {
        for input in transaction.inputs() {
            self.remove(input)
        }
        for (vout, output) in transaction.outputs().iter().enumerate() {
            if output.public_key() != self.public_key() {
                continue;
            }
            let utxo = Utxo::new(
                UtxoId::new(*transaction.id(), vout),
                UtxoData::new(output.amount(), *output.public_key()),
            );
            self.add(utxo);
        }
    }

    pub fn process_transactions_from(&mut self, block: &Block) {}

    pub fn undo_transactions_from(&mut self, block: &Block) {}

    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    pub fn utxos(&self) -> &Vec<Utxo> {
        &self.utxos
    }
}
