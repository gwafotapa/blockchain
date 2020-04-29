use secp256k1::PublicKey;
use std::collections::HashMap;
use std::fmt;
use std::result;

use super::{Utxo, UtxoData, UtxoId};
use crate::common::{Hash, INIT_UTXO_AMOUNT, INIT_UTXO_HASH};
use crate::transaction::{InvalidTransaction, Transaction, TransactionInput, TransactionOutput};

#[derive(Debug)]
pub struct UtxoPool {
    data: HashMap<UtxoId, UtxoData>,
}

impl UtxoPool {
    pub fn new(keys: Vec<PublicKey>) -> Self {
        Self {
            data: keys
                .into_iter()
                .enumerate()
                .map(|(n, pk)| {
                    (
                        UtxoId::new(Hash::from(INIT_UTXO_HASH), n),
                        UtxoData::new(INIT_UTXO_AMOUNT, pk),
                    )
                })
                .collect(),
        }
    }

    pub fn add(&mut self, utxo: Utxo) {
        self.data.insert(utxo.id, utxo.data);
    }

    pub fn remove(&mut self, utxo: &Utxo) -> Option<UtxoData> {
        self.data.remove(utxo.id())
    }

    pub fn contains(&self, utxo: Utxo) -> bool {
        self.data.contains_key(utxo.id())
    }

    // pub fn find(&self, input: &TransactionInput) -> Option<UtxoData> {
    //     self.data.contains_key(UtxoId::new(input.id(), input.vout())
    // }

    pub fn owned_by(&self, pk: &PublicKey) -> Vec<Utxo> {
        self.data
            .iter()
            .filter(|(_id, data)| data.public_key() == pk)
            .map(|(id, data)| Utxo::new(id.clone(), data.clone()))
            .collect()
    }

    pub fn process(&mut self, transaction: &Transaction) -> result::Result<(), InvalidTransaction> {
        for input in transaction.inputs() {
            if !self.data.contains_key(input.utxo_id()) {
                return Err(InvalidTransaction);
            }
        }
        for input in transaction.inputs() {
            self.data.remove(input.utxo_id());
        }
        for (vout, output) in transaction.outputs().iter().enumerate() {
            let utxo_id = UtxoId::new(*transaction.id(), vout);
            let utxo_data = UtxoData::new(output.amount(), *output.public_key());
            self.data.insert(utxo_id, utxo_data);
        }
        Ok(())
    }
}

// impl Eq for UtxoPool {}

// impl PartialEq for UtxoPool {
//     fn eq(&self, other: &Self) -> bool {
//         let (p1, _): (HashSet<TransactionInput>, HashSet<TransactionOutput>) =
//             self.data.iter().unzip();
//         let (p2, _): (HashSet<TransactionInput>, HashSet<TransactionOutput>) =
//             other.data.iter().unzip();
//         p1.symmetric_difference(&p2).next().is_none()
//     }
// }

impl fmt::Display for UtxoPool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (utxo_id, utxo_data) in &self.data {
            write!(f, "{}\n{}\n\n", utxo_id, utxo_data)?;
        }
        Ok(())
    }
}
