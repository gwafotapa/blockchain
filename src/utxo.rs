// use rand::Rng;
use std::collections::HashMap;
use std::convert::TryInto;
use std::{fmt, result};

use crate::transaction::{InvalidTransaction, Transaction};

/// Amount of initial utxos
const INIT_AMOUNT: u32 = 10;

/// For now, a utxo has an owner (instead of a script that someone has the unlocking key for)
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Utxo {
    amount: u32,
    puzzle: usize, // for now this is just a node number (supposedly having the unlocking key)
}

// impl PartialEq for Utxo {
//     fn eq(&self, other: &Self) -> bool {
//         self.puzzle == other.puzzle
//     }
// }

impl From<&[u8]> for Utxo {
    fn from(bytes: &[u8]) -> Self {
        let amount = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
        let puzzle = usize::from_be_bytes(bytes[4..12].try_into().unwrap());
        Self { amount, puzzle }
    }
}

impl Utxo {
    pub fn new(amount: u32, puzzle: usize) -> Self {
        Self { amount, puzzle }
    }

    pub fn amount(&self) -> u32 {
        self.amount
    }

    pub fn puzzle(&self) -> usize {
        self.puzzle
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.amount
            .to_be_bytes()
            .iter()
            .chain(self.puzzle.to_be_bytes().iter())
            .copied()
            .collect()
    }
}

impl fmt::Display for Utxo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Amount: {:>3}\tOwner: {:>2}", self.amount, self.puzzle)
    }
}

type Node = usize;

pub struct UtxoPool {
    data: HashMap<Node, Vec<Utxo>>,
}

impl UtxoPool {
    pub fn new(nodes: usize) -> Self {
        Self {
            data: (0..nodes)
                .into_iter()
                .map(|x| (x, vec![Utxo::new(INIT_AMOUNT, x)]))
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
            .and_then(|v| Some(v.remove(v.iter().position(|x| x.puzzle == utxo.puzzle).unwrap())))
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
        if self.remove(transaction.input()) {
            for output in transaction.outputs() {
                self.add(*output);
            }
            Ok(())
        } else {
            Err(InvalidTransaction)
        }
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
