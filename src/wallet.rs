use rand::seq::IteratorRandom;
use rand::Rng;
use secp256k1::{Message as MessageToSign, PublicKey, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};
use std::fmt;

use crate::chain::Blockchain;
use crate::common::SPEND_PROBA;
use crate::transaction::{Transaction, TransactionInput, TransactionOutput};
use crate::utxo::{Utxo, UtxoData, UtxoId};

pub struct Wallet {
    public_key: PublicKey,
    secret_key: SecretKey,
    recipients: Vec<PublicKey>,
    utxos: Vec<Utxo>,
}

impl Wallet {
    pub fn new(
        public_key: PublicKey,
        secret_key: SecretKey,
        recipients: Vec<PublicKey>,
        utxos: Vec<Utxo>,
    ) -> Self {
        Self {
            public_key,
            secret_key,
            recipients,
            utxos,
        }
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
            .position(|utxo| utxo.id() == input.utxo_id())
            .and_then(|i| Some(self.utxos.remove(i)));
    }

    pub fn initiate(&mut self) -> Option<Transaction> {
        if self.utxos().is_empty() {
            return None;
        }
        let mut rng = rand::thread_rng();
        match rng.gen_bool(SPEND_PROBA) {
            false => None,
            true => {
                let inputs_len = rng.gen_range(1, self.utxos().len() + 1);
                let utxos = self.utxos().iter().choose_multiple(&mut rng, inputs_len);
                let mut amount: u32 = utxos.iter().map(|u| u.amount()).sum();
                let mut outputs = Vec::new();
                loop {
                    let amount1 = rng.gen_range(1, amount + 1);
                    let recipient = *self.recipients.iter().choose(&mut rng).unwrap();
                    let output = TransactionOutput::new(amount1, recipient);
                    outputs.push(output);
                    amount -= amount1;
                    if amount == 0 {
                        break;
                    }
                }
                let mut message = Vec::new();
                for utxo in &utxos {
                    message.extend(utxo.id().serialize());
                }
                for output in &outputs {
                    message.extend(output.serialize());
                }
                let mut hasher = Sha256::new();
                hasher.input(message);
                let hash = hasher.result();
                let message = MessageToSign::from_slice(&hash).unwrap();
                let secp = Secp256k1::new();
                let sig = secp.sign(&message, &self.secret_key);
                let inputs = utxos
                    .iter()
                    .map(|u| TransactionInput::new(*u.id(), sig))
                    .collect();
                let transaction = Transaction::new(inputs, outputs);
                Some(transaction)
            }
        }
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
                UtxoId::new(transaction.id(), vout),
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
                UtxoId::new(transaction.id(), vout),
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

impl fmt::Display for Wallet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Wallet ({}) {{", self.utxos.len())?;
        for utxo in &self.utxos {
            write!(
                f,
                "\n  txid: {}  vout:{}\n  public_key: {}  amount: {}\n",
                format!("{:#x}", utxo.txid()),
                utxo.vout(),
                utxo.public_key(),
                utxo.amount()
            )?;
        }
        write!(f, "}}\n")
    }
}
