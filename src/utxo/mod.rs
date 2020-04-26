use std::fmt;

use crate::common::Hash;
use crate::transaction::{TransactionInput, TransactionOutput};

/// For now, a utxo has an owner (instead of a script that someone has the unlocking key for)
// #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Utxo {
    input: TransactionInput,
    output: TransactionOutput,
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
    // pub fn new(txid: Hash, vout: usize, amount: u32, puzzle: usize) -> Self {
    //     Self {
    //         txid,
    //         vout,
    //         amount,
    //         puzzle,
    //     }
    // }

    pub fn new(input: TransactionInput, output: TransactionOutput) -> Self {
        Self { input, output }
    }

    pub fn input(&self) -> &TransactionInput {
        &self.input
    }

    pub fn output(&self) -> &TransactionOutput {
        &self.output
    }

    pub fn txid(&self) -> Hash {
        self.input.txid()
    }

    pub fn vout(&self) -> usize {
        self.input.vout()
    }

    pub fn amount(&self) -> u32 {
        self.output.amount()
    }

    pub fn puzzle(&self) -> usize {
        self.output.puzzle()
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
            "txid: {:?}\n\
             vout: {}\n\
             amount: {}\n\
             puzzle: {}",
            self.txid(),
            self.vout(),
            self.amount(),
            self.puzzle()
        )
    }
}

pub mod pool;
