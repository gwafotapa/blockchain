use secp256k1::PublicKey;

use crate::block::Block;
use crate::chain::Blockchain;
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

    // TODO: add or push ? Harmonize between all files
    pub fn add(&mut self, utxo: Utxo) {
        if utxo.public_key() != self.public_key() {
            panic!("Invalid public key")
        }
        self.utxos.push(utxo);
    }

    pub fn remove(&mut self, utxo: &Utxo) {
        self.utxos
            .iter()
            .position(|u| u == utxo)
            .and_then(|i| Some(self.utxos.remove(i)));
    }

    pub fn remove_utxo_from(&mut self, input: &TransactionInput) {
        self.utxos
            .iter()
            .position(|utxo| utxo.txid() == input.txid() && utxo.vout() == input.vout())
            .and_then(|i| Some(self.utxos.remove(i)));
    }

    pub fn process(&mut self, transaction: &Transaction) {
        for input in transaction.inputs() {
            self.remove_utxo_from(input)
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

    pub fn process_all(&mut self, transactions: &[Transaction]) {
        for transaction in transactions {
            self.process(transaction);
        }
    }

    pub fn undo(&mut self, transaction: &Transaction, blockchain: &Blockchain) {
        for input in transaction.inputs() {
            let utxo = blockchain.get_utxo_from(input);
            self.add(utxo);
        }
        for (vout, output) in transaction.outputs().iter().enumerate() {
            if output.public_key() != self.public_key() {
                continue;
            }
            let utxo = Utxo::new(
                UtxoId::new(*transaction.id(), vout),
                UtxoData::new(output.amount(), *output.public_key()),
            );
            self.remove(&utxo);
        }
    }

    pub fn undo_all(&mut self, transactions: &[Transaction], blockchain: &Blockchain) {
        // TODO: shouldn't I loop in reverse order ?? (same in utxo pool)
        for transaction in transactions {
            self.undo(transaction, blockchain);
        }
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    pub fn utxos(&self) -> &Vec<Utxo> {
        &self.utxos
    }
}
