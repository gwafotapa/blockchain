use std::collections::{HashMap, HashSet};
use std::fmt;
use std::result;

use super::Utxo;
use crate::common::{Hash, INIT_UTXO_AMOUNT, INIT_UTXO_HASH};
use crate::transaction::{InvalidTransaction, Transaction, TransactionInput, TransactionOutput};

pub struct UtxoPool {
    data: HashMap<TransactionInput, TransactionOutput>,
}

impl UtxoPool {
    pub fn new(nodes: usize) -> Self {
        Self {
            data: (0..nodes)
                .into_iter()
                .map(|x| {
                    (
                        TransactionInput::new(Hash::from(INIT_UTXO_HASH), x),
                        TransactionOutput::new(INIT_UTXO_AMOUNT, x),
                    )
                })
                .collect(),
        }
    }

    pub fn add(&mut self, utxo: Utxo) {
        // self.data.get_mut(&utxo.puzzle).unwrap().push(utxo);
        self.data.insert(*utxo.input(), *utxo.output());
    }

    pub fn remove(&mut self, utxo: &Utxo) -> bool {
        // self.data
        //     .get_mut(&utxo.puzzle)
        //     .and_then(|v| Some(v.remove(v.iter().position(|x| x.amount == utxo.amount).unwrap())))
        //     .is_some()
        self.data.remove(utxo.input()).is_some()
    }

    // pub fn contains(&self, utxo: &Utxo) -> bool {
    //     self.data.contains_key(&utxo.puzzle)
    // }

    pub fn contains(&self, utxo: &Utxo) -> bool {
        self.data.contains_key(utxo.input())
    }

    // pub fn random(&self) -> Utxo {
    //     let mut rng = rand::thread_rng();
    //     let n = rng.gen_range(0, self.data.keys().len());
    //     let puzzle = *self.data.keys().nth(n).unwrap();
    //     let amount = *self.data.get(&puzzle).unwrap();
    //     Utxo { amount, puzzle }
    // }

    pub fn node(&self, node: usize) -> Vec<Utxo> {
        self.data
            .iter()
            .filter(|(_i, o)| o.puzzle() == node)
            .map(|(i, o)| Utxo::new(*i, *o))
            .collect()
    }

    pub fn process(&mut self, transaction: &Transaction) -> result::Result<(), InvalidTransaction> {
        for input in transaction.inputs() {
            if !self.data.contains_key(input) {
                return Err(InvalidTransaction);
            }
        }
        for input in transaction.inputs() {
            self.data.remove(input);
        }
        let mut vout = 0;
        for &output in transaction.outputs() {
            let input = TransactionInput::new(transaction.id(), vout);
            self.data.insert(input, output);
            vout += 1;
        }
        // let indices = Vec::new();
        // for input in transaction.inputs() {
        //     match self.find(input) {
        //         None => return InvalidTransaction,
        //         Some(utxo) =>

        // let id = transaction.inputs()[0].puzzle();
        // let utxos: HashSet<Utxo> = self.data[&id].iter().copied().collect();
        // let inputs: HashSet<Utxo> = transaction.inputs().iter().copied().collect();
        // if !inputs.is_subset(&utxos) {
        //     return Err(InvalidTransaction);
        // }
        // for input in transaction.inputs() {
        //     self.remove(*input);
        // }
        // for output in transaction.outputs() {
        //     self.add(*output);
        // }
        Ok(())
    }
}

impl Eq for UtxoPool {}

impl PartialEq for UtxoPool {
    fn eq(&self, other: &Self) -> bool {
        let (p1, _): (HashSet<TransactionInput>, HashSet<TransactionOutput>) =
            self.data.iter().unzip();
        let (p2, _): (HashSet<TransactionInput>, HashSet<TransactionOutput>) =
            other.data.iter().unzip();
        p1.symmetric_difference(&p2).next().is_none()
    }
}

impl fmt::Display for UtxoPool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (_input, output) in &self.data {
            write!(f, " ({}, {}) ", output.puzzle(), output.amount())?;
        }
        Ok(())
    }
}

impl fmt::Debug for UtxoPool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (_input, output) in &self.data {
            write!(f, " ({}, {}) ", output.puzzle(), output.amount())?;
        }
        Ok(())
    }
}
