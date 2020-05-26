use rand::seq::IteratorRandom;
use rand::Rng;
use secp256k1::{PublicKey, SecretKey};
use std::collections::HashSet;
use std::fmt;

use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::constants::{SPEND_PROBA, UTXO_HASH_INIT};
use crate::error::wallet::WalletError;
use crate::transaction::{Transaction, TransactionInput, TransactionOutput};
use crate::utxo::{Utxo, UtxoData, UtxoId};
use crate::utxo_pool::UtxoPool;
use crate::Hash;

pub struct Wallet {
    public_key: PublicKey,
    secret_key: SecretKey,
    recipients: Vec<PublicKey>,
    utxos: HashSet<Utxo>,
}

impl Wallet {
    pub fn new(
        public_key: PublicKey,
        secret_key: SecretKey,
        recipients: Vec<PublicKey>,
        utxos: HashSet<Utxo>,
    ) -> Self {
        Self {
            public_key,
            secret_key,
            recipients,
            utxos,
        }
    }

    pub fn add(&mut self, utxo: Utxo) -> Result<(), WalletError> {
        if utxo.public_key() != self.public_key() {
            Err(WalletError::WrongPublicKey)
        } else {
            if self.utxos.insert(utxo) {
                Ok(())
            } else {
                Err(WalletError::KnownUtxo)
            }
        }
    }

    pub fn remove(&mut self, utxo: &Utxo) -> Result<(), WalletError> {
        if self.utxos.remove(utxo) {
            Ok(())
        } else {
            Err(WalletError::UnknownUtxo)
        }
    }

    pub fn remove_if_utxo_from(&mut self, input: &TransactionInput) -> bool {
        if let Some(utxo) = self
            .utxos
            .iter()
            .filter(|utxo| utxo.id() == input.utxo_id())
            .copied()
            .last()
        {
            self.remove(&utxo).is_ok()
        } else {
            false
        }
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
                let utxo_ids = utxos.iter().map(|u| *u.utxo_id()).collect();
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
                let transaction = Transaction::sign(utxo_ids, outputs, &self.secret_key);
                Some(transaction)
            }
        }
    }

    pub fn double_spend(&mut self) -> Option<(Transaction, Transaction)> {
        if self.utxos().is_empty() || self.recipients.len() < 2 {
            return None;
        }
        let mut rng = rand::thread_rng();
        match rng.gen_bool(SPEND_PROBA) {
            false => None,
            true => {
                let utxo = self.utxos().iter().choose(&mut rng).unwrap();
                let utxo_ids = vec![*utxo.utxo_id()];
                let recipients = self.recipients.iter().choose_multiple(&mut rng, 2);

                let output1 = TransactionOutput::new(utxo.amount(), *recipients[0]);
                let outputs1 = vec![output1];
                let transaction1 = Transaction::sign(utxo_ids.clone(), outputs1, &self.secret_key);

                let output2 = TransactionOutput::new(utxo.amount(), *recipients[1]);
                let outputs2 = vec![output2];
                let transaction2 = Transaction::sign(utxo_ids, outputs2, &self.secret_key);

                Some((transaction1, transaction2))
            }
        }
    }

    pub fn process_t(&mut self, transaction: &Transaction) {
        for input in transaction.inputs() {
            self.remove_if_utxo_from(input);
        }
        for (vout, output) in transaction.outputs().iter().enumerate() {
            if output.public_key() != self.public_key() {
                continue;
            }
            let utxo = Utxo::new(
                UtxoId::new(*transaction.id(), vout),
                UtxoData::new(output.amount(), *output.public_key()),
            );
            self.add(utxo).unwrap();
        }
    }

    pub fn process(&mut self, block: &Block) {
        for transaction in block.transactions() {
            self.process_t(transaction);
        }
    }

    pub fn process_all(&mut self, blocks: &[Block]) {
        for block in blocks {
            self.process(block);
        }
    }

    pub fn undo_t(
        &mut self,
        transaction: &Transaction,
        blockchain: &Blockchain,
        utxo_pool: &UtxoPool,
    ) {
        for (vout, output) in transaction.outputs().iter().enumerate() {
            if output.public_key() != self.public_key() {
                continue;
            }
            let utxo = Utxo::new(
                UtxoId::new(*transaction.id(), vout),
                UtxoData::new(output.amount(), *output.public_key()),
            );
            self.remove(&utxo).unwrap();
        }

        for input in transaction.inputs() {
            if *input.txid() == Hash::from(UTXO_HASH_INIT) {
                let utxo_id = UtxoId::new(*input.txid(), input.vout());
                let utxo_data = utxo_pool.initial_utxos()[&utxo_id];
                let utxo = Utxo::new(utxo_id, utxo_data);
                if utxo.public_key() == self.public_key() {
                    self.add(utxo).unwrap();
                }
            } else {
                let utxo = blockchain.get_utxo(input.utxo_id(), blockchain.top());
                if utxo.public_key() == self.public_key() {
                    self.add(utxo).unwrap();
                }
            }
        }
    }

    pub fn undo(&mut self, block: &Block, blockchain: &Blockchain, utxo_pool: &UtxoPool) {
        for transaction in block.transactions() {
            self.undo_t(transaction, blockchain, utxo_pool);
        }
    }

    pub fn undo_all(&mut self, blocks: &[Block], blockchain: &Blockchain, utxo_pool: &UtxoPool) {
        for block in blocks.iter().rev() {
            self.undo(block, blockchain, utxo_pool);
        }
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    pub fn utxos(&self) -> &HashSet<Utxo> {
        &self.utxos
    }
}

impl fmt::Display for Wallet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Wallet ({}) {{", self.utxos.len())?;
        for utxo in &self.utxos {
            write!(
                f,
                "\n  txid: {:x}  vout:{}\n  public_key: {}  amount: {}\n",
                utxo.txid(),
                utxo.vout(),
                utxo.public_key(),
                utxo.amount()
            )?;
        }
        write!(f, "}}\n")
    }
}
