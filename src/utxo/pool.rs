use secp256k1::{Message as MessageToSign, PublicKey, Secp256k1};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fmt;

use super::{Utxo, UtxoData, UtxoId};
use crate::block::{Block, BlockError};
use crate::blockchain::Blockchain;
use crate::common::{Hash, UTXO_AMOUNT_INIT, UTXO_HASH_INIT};
use crate::transaction::{Transaction, TransactionError};

#[derive(Clone, Debug)]
pub struct UtxoPool {
    utxos: HashMap<UtxoId, UtxoData>,
    initial_utxos: HashMap<UtxoId, UtxoData>,
}

impl UtxoPool {
    pub fn new(keys: Vec<PublicKey>) -> Self {
        let utxos: HashMap<UtxoId, UtxoData> = keys
            .into_iter()
            .enumerate()
            .map(|(n, pk)| {
                (
                    UtxoId::new(Hash::from(UTXO_HASH_INIT), n),
                    UtxoData::new(UTXO_AMOUNT_INIT, pk),
                )
            })
            .collect();
        let initial_utxos = utxos.clone();
        Self {
            utxos,
            initial_utxos,
        }
    }

    pub fn add(&mut self, utxo: Utxo) {
        self.utxos.insert(utxo.id, utxo.data);
    }

    pub fn remove(&mut self, utxo: &Utxo) -> Option<UtxoData> {
        self.utxos.remove(utxo.id())
    }

    pub fn contains(&self, utxo: Utxo) -> bool {
        self.utxos.contains_key(utxo.id())
    }

    pub fn owned_by(&self, pk: &PublicKey) -> Vec<Utxo> {
        self.utxos
            .iter()
            .filter(|(_id, data)| data.public_key() == pk)
            .map(|(id, data)| Utxo::new(id.clone(), data.clone()))
            .collect()
    }

    pub fn process_t(&mut self, transaction: &Transaction) {
        for input in transaction.inputs() {
            self.utxos.remove(input.utxo_id());
        }
        for (vout, output) in transaction.outputs().iter().enumerate() {
            let utxo_id = UtxoId::new(transaction.id(), vout);
            let utxo_data = UtxoData::new(output.amount(), *output.public_key());
            // TODO: use self.add instead and look in the rest of the file for such things
            self.utxos.insert(utxo_id, utxo_data);
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

    pub fn undo_t(&mut self, transaction: &Transaction, blockchain: &Blockchain, block: &Block) {
        for input in transaction.inputs() {
            if input.txid() == Hash::from(UTXO_HASH_INIT) {
                let utxo_id = UtxoId::new(input.txid(), input.vout());
                let utxo_data = self.initial_utxos[&utxo_id];
                let utxo = Utxo::new(utxo_id, utxo_data);
                self.add(utxo);
            } else {
                let utxo = blockchain.get_utxo(input.utxo_id(), block);
                self.add(utxo);
            }
        }
        for (vout, output) in transaction.outputs().iter().enumerate() {
            let utxo_id = UtxoId::new(transaction.id(), vout);
            let utxo_data = UtxoData::new(output.amount(), *output.public_key());
            let utxo = Utxo::new(utxo_id, utxo_data);
            self.remove(&utxo)
                .ok_or("Utxo cannot be removed from the pool")
                .unwrap();
        }
    }

    pub fn undo(&mut self, block: &Block, blockchain: &Blockchain) {
        for transaction in block.transactions() {
            self.undo_t(transaction, blockchain, block);
        }
    }

    pub fn undo_all(&mut self, blocks: &[Block], blockchain: &Blockchain) {
        for block in blocks.iter().rev() {
            self.undo(block, blockchain);
        }
    }

    // TODO: Need to check each input is only used once
    pub fn verify(&self, transaction: &Transaction) -> Result<(), TransactionError> {
        let mut message = Vec::new();
        for utxo_id in transaction.inputs().iter().map(|i| i.utxo_id()) {
            message.extend(utxo_id.serialize());
        }
        for output in transaction.outputs() {
            message.extend(output.serialize());
        }
        let mut hasher = Sha256::new();
        hasher.input(message);
        let hash = hasher.result();
        let message = MessageToSign::from_slice(&hash).unwrap();
        let secp = Secp256k1::new();
        for input in transaction.inputs() {
            if let Some(utxo_data) = self.utxos.get(input.utxo_id()) {
                secp.verify(&message, input.sig(), utxo_data.public_key())?;
            } else {
                return Err(TransactionError::UnknownUtxo);
            }
        }
        Ok(())
    }

    // TODO: Need to check no two transactions share an input
    pub fn validate(&self, block: &Block) -> Result<(), BlockError> {
        if !block.transaction_count().is_power_of_two() {
            return Err(BlockError::WrongTransactionCount);
        }
        for transaction in block.transactions() {
            self.verify(transaction)?;
        }
        Ok(())
    }

    pub fn size(&self) -> usize {
        self.utxos.len()
    }

    pub fn initial_utxos(&self) -> &HashMap<UtxoId, UtxoData> {
        &self.initial_utxos
    }

    pub fn utxos(&self) -> &HashMap<UtxoId, UtxoData> {
        &self.utxos
    }
}

impl Eq for UtxoPool {}

impl PartialEq for UtxoPool {
    fn eq(&self, other: &Self) -> bool {
        let p1: HashSet<UtxoId> = self.utxos.iter().map(|(id, _)| id).copied().collect();
        let p2: HashSet<UtxoId> = other.utxos.iter().map(|(id, _)| id).copied().collect();
        p1.symmetric_difference(&p2).next().is_none()
    }
}

impl fmt::Display for UtxoPool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Utxo pool ({}) {{", self.size())?;
        for (utxo_id, utxo_data) in &self.utxos {
            write!(
                f,
                "\n  txid: {}  vout:{}\n  public_key: {}  amount: {}\n",
                format!("{:#x}", utxo_id.txid()),
                utxo_id.vout(),
                utxo_data.public_key(),
                utxo_data.amount()
            )?;
        }
        write!(f, "}}\n")
    }
}
