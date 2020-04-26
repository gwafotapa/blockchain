// use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::{fmt, result};

use crate::transaction::{InvalidTransaction, Transaction};

/// Amount of initial utxos
const INIT_AMOUNT: u32 = 10;
const INIT_HASH: Hash = Hash::from([0u8; 32]);

/// For now, a utxo has an owner (instead of a script that someone has the unlocking key for)
// #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Utxo {
    txid: Hash,
    vout: usize,
    amount: u32,
    puzzle: usize, // for now this is just a node number (supposedly having the unlocking key)
}

// impl PartialEq for Utxo {
//     fn eq(&self, other: &Self) -> bool {
//         self.puzzle == other.puzzle
//     }
// }

// impl From<&[u8]> for Utxo {
//     fn from(bytes: &[u8]) -> Self {
//         let amount = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
//         let puzzle = usize::from_be_bytes(bytes[4..12].try_into().unwrap());
//         Self { amount, puzzle }
//     }
// }

impl Utxo {
    pub fn new(txid: Hash, vout: usize, amount: u32, puzzle: usize) -> Self {
        Self {
            txid,
            vout,
            amount,
            puzzle,
        }
    }

    pub fn txid(&self) -> Hash {
        self.txid
    }

    pub fn vout(&self) -> usize {
        self.vout
    }

    pub fn amount(&self) -> u32 {
        self.amount
    }

    pub fn puzzle(&self) -> usize {
        self.puzzle
    }

    // pub fn serialize(&self) -> Vec<u8> {
    //     self.amount
    //         .to_be_bytes()
    //         .iter()
    //         .chain(self.puzzle.to_be_bytes().iter())
    //         .copied()
    //         .collect()
    // }
}

impl fmt::Display for Utxo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "txid: {}\n\
             vout: {}\n\
             amount: {}\n\
             puzzle: {}",
            self.txid, self.vout, self.amount, self.puzzle
        )
    }
}

pub struct UtxoPool {
    data: HashMap<TransactionInput, TransactionOutput>,
}

impl UtxoPool {
    pub fn new(nodes: usize) -> Self {
        Self {
            data: (0..nodes)
                .into_iter()
                .map(|x| (x, vec![TransactionInput::new(INIT_HASH, x), INIT_AMOUNT, x)]))
                .collect(),
        }
    }

    pub fn node(&self, node: usize) -> &[Utxo] {
        &self.data[&node]
    }

    pub fn add(&mut self, utxo: Utxo) {
        self.data.get_mut(&utxo.puzzle).unwrap().push(utxo);
    }

    pub fn remove(&mut self, utxo: Utxo) -> bool {
        self.data
            .get_mut(&utxo.puzzle)
            .and_then(|v| Some(v.remove(v.iter().position(|x| x.amount == utxo.amount).unwrap())))
            .is_some()
    }

    // pub fn contains(&self, utxo: &Utxo) -> bool {
    //     self.data.contains_key(&utxo.puzzle)
    // }

    // pub fn random(&self) -> Utxo {
    //     let mut rng = rand::thread_rng();
    //     let n = rng.gen_range(0, self.data.keys().len());
    //     let puzzle = *self.data.keys().nth(n).unwrap();
    //     let amount = *self.data.get(&puzzle).unwrap();
    //     Utxo { amount, puzzle }
    // }

    pub fn process(&mut self, transaction: &Transaction) -> result::Result<(), InvalidTransaction> {
        // TODO: rewrite
        let indices = Vec::new();
        for input in transaction.inputs() {
            match self.find(input) {
                None => return InvalidTransaction,
                Some(utxo) => 
            
        let id = transaction.inputs()[0].puzzle();
        let utxos: HashSet<Utxo> = self.data[&id].iter().copied().collect();
        let inputs: HashSet<Utxo> = transaction.inputs().iter().copied().collect();
        if !inputs.is_subset(&utxos) {
            return Err(InvalidTransaction);
        }
        for input in transaction.inputs() {
            self.remove(*input);
        }
        for output in transaction.outputs() {
            self.add(*output);
        }
        Ok(())
    }
}

impl fmt::Display for UtxoPool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Utxo pool:\n")?;
        for (node, utxos) in &self.data {
            write!(f, "Node #{:>2}:", node)?;
            for utxo in utxos {
                write!(f, " {:>3}", utxo.amount())?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}
