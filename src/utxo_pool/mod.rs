use secp256k1::{Message as MessageToSign, PublicKey, Secp256k1};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::constants::{UTXO_AMOUNT_INIT, UTXO_HASH_INIT};
use crate::error::utxo_pool::UtxoPoolError;
use crate::transaction::Transaction;
use crate::utxo::{Utxo, UtxoData, UtxoId};
use crate::Hash;

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

    pub fn add(&mut self, utxo: Utxo) -> Result<(), UtxoPoolError> {
        match self.utxos.insert(*utxo.id(), *utxo.data()) {
            None => Ok(()),
            Some(_) => Err(UtxoPoolError::KnownUtxo),
        }
    }

    pub fn remove(&mut self, utxo: &Utxo) -> Result<UtxoData, UtxoPoolError> {
        self.utxos
            .remove(utxo.id())
            .ok_or(UtxoPoolError::UnknownUtxo)
    }

    pub fn contains(&self, utxo: &Utxo) -> bool {
        self.utxos.contains_key(utxo.id())
    }

    pub fn owned_by(&self, pk: &PublicKey) -> HashSet<Utxo> {
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
            let utxo_id = UtxoId::new(*transaction.id(), vout);
            let utxo_data = UtxoData::new(output.amount(), *output.public_key());
            let utxo = Utxo::new(utxo_id, utxo_data);
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

    pub fn undo_t(&mut self, transaction: &Transaction, blockchain: &Blockchain, block: &Block) {
        for (vout, output) in transaction.outputs().iter().enumerate() {
            let utxo_id = UtxoId::new(*transaction.id(), vout);
            let utxo_data = UtxoData::new(output.amount(), *output.public_key());
            let utxo = Utxo::new(utxo_id, utxo_data);
            self.remove(&utxo).unwrap();
        }

        for input in transaction.inputs() {
            if *input.txid() == Hash::from(UTXO_HASH_INIT) {
                let utxo_id = UtxoId::new(*input.txid(), input.vout());
                let utxo_data = self.initial_utxos[&utxo_id];
                let utxo = Utxo::new(utxo_id, utxo_data);
                self.add(utxo).unwrap();
            } else {
                let utxo = blockchain.get_utxo(input.utxo_id(), block);
                self.add(utxo).unwrap();
            }
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

    // pub fn verify(&self, transaction: &Transaction) -> Result<(), UtxoPoolError> {
    //     let input_utxos: HashSet<_> = transaction.inputs().iter().map(|i| *i.utxo_id()).collect();
    //     if input_utxos.len() != transaction.inputs().len() {
    //         return Err(UtxoPoolError::TransactionHasDoubleSpending);
    //     }
    //     self.authenticate(transaction)
    // }

    // pub fn authenticate(&self, transaction: &Transaction) -> Result<(), UtxoPoolError> {
    //     let mut message = Vec::new();
    //     for utxo_id in transaction.inputs().iter().map(|i| i.utxo_id()) {
    //         message.extend(utxo_id.serialize());
    //     }
    //     for output in transaction.outputs() {
    //         message.extend(output.serialize());
    //     }
    //     let mut hasher = Sha256::new();
    //     hasher.input(message);
    //     let hash = hasher.result();
    //     let message = MessageToSign::from_slice(&hash).unwrap();
    //     let secp = Secp256k1::new();
    //     for input in transaction.inputs() {
    //         if let Some(utxo_data) = self.utxos.get(input.utxo_id()) {
    //             secp.verify(&message, input.sig(), utxo_data.public_key())?;
    //         } else {
    //             return Err(UtxoPoolError::TransactionHasUnknownUtxo);
    //         }
    //     }
    //     Ok(())
    // }

    pub fn recalculate(
        &mut self,
        blocks_to_undo: &Vec<Block>,
        blocks_to_process: &Vec<Block>,
        blockchain: &Blockchain,
    ) {
        self.undo_all(blocks_to_undo, blockchain);
        self.process_all(blocks_to_process);
    }

    pub fn check_utxos_exist_for(&self, transaction: &Transaction) -> Result<(), UtxoPoolError> {
        for input in transaction.inputs() {
            self.utxos
                .get(input.utxo_id())
                .ok_or(UtxoPoolError::TransactionHasUnknownUtxo)?;
        }
        Ok(())
    }

    // pub fn check_double_spending(&self, transaction: &Transaction) -> Result<(), UtxoPoolError> {
    //     let input_utxos: HashSet<_> = transaction.inputs().iter().map(|i| *i.utxo_id()).collect();
    //     if input_utxos.len() != transaction.inputs().len() {
    //         return Err(UtxoPoolError::TransactionHasDoubleSpending);
    //     }
    //     Ok(())
    // }

    pub fn authenticate(&self, transaction: &Transaction) -> Result<(), UtxoPoolError> {
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
            }
        }
        Ok(())
    }

    pub fn check_utxos_exist(&self, block: &Block) -> Result<(), UtxoPoolError> {
        for transaction in block.transactions() {
            self.check_utxos_exist_for(transaction)?;
        }
        Ok(())
    }

    pub fn check_signatures_of(&self, block: &Block) -> Result<(), UtxoPoolError> {
        for transaction in block.transactions() {
            self.authenticate(transaction)?;
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
                "\n  txid: {:x}  vout:{}\n  public_key: {}  amount: {}\n",
                utxo_id.txid(),
                utxo_id.vout(),
                utxo_data.public_key(),
                utxo_data.amount()
            )?;
        }
        write!(f, "}}\n")
    }
}

impl From<(HashMap<UtxoId, UtxoData>, HashMap<UtxoId, UtxoData>)> for UtxoPool {
    fn from(utxos: (HashMap<UtxoId, UtxoData>, HashMap<UtxoId, UtxoData>)) -> Self {
        Self {
            utxos: utxos.0,
            initial_utxos: utxos.1,
        }
    }
}

impl From<(HashSet<Utxo>, HashSet<Utxo>)> for UtxoPool {
    fn from(utxos: (HashSet<Utxo>, HashSet<Utxo>)) -> Self {
        let initial_utxos = utxos.1.iter().map(|u| (*u.id(), *u.data())).collect();
        let utxos = utxos.0.iter().map(|u| (*u.id(), *u.data())).collect();
        Self {
            utxos,
            initial_utxos,
        }
    }
}
