use merkle_cbt::merkle_tree::CBMT;
// use rand::Rng;
use sha2::{Digest, Sha256};
// use std::convert::TryInto;
use std::error::Error;
use std::fmt;

use self::merkle_tree::MergeHash;
use crate::common::Hash;
use crate::utxo::Utxo;

pub use pool::TransactionPool;

const SIZE_BYTES: usize = 12;

#[derive(Debug, Clone)]
pub struct InvalidTransaction;

impl fmt::Display for InvalidTransaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid transaction")
    }
}

impl Error for InvalidTransaction {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transaction {
    input: Utxo,
    outputs: Vec<Utxo>,
}

impl Transaction {
    pub fn new(input: Utxo, outputs: Vec<Utxo>) -> Self {
        Self { input, outputs }
    }

    pub fn input(&self) -> Utxo {
        self.input
    }

    pub fn outputs(&self) -> &[Utxo] {
        &self.outputs
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.input
            .serialize()
            .into_iter()
            .chain(self.outputs.iter().flat_map(|o| o.serialize()))
            .collect()
    }

    pub fn deserialize<B>(bytes: B) -> Self
    where
        B: AsRef<[u8]>,
    {
        Transaction::from(bytes.as_ref())
    }

    pub fn hash(&self) -> Hash {
        let mut hasher = Sha256::new();
        hasher.input(self.serialize());
        hasher.result()
    }

    pub fn hash_merkle_root(transactions: &Vec<Self>) -> Hash {
        let hashes = transactions.iter().map(|x| x.hash()).collect();
        let merkle_tree = CBMT::<Hash, MergeHash>::build_merkle_tree(hashes);
        merkle_tree.root()
    }
}

impl From<&[u8]> for Transaction {
    fn from(bytes: &[u8]) -> Self {
        let input = Utxo::from(&bytes[0..SIZE_BYTES]);
        let outputs = bytes[SIZE_BYTES..]
            .chunks_exact(SIZE_BYTES)
            .map(|c| Utxo::from(c))
            .collect();
        Self { input, outputs }
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Sender:     {:>2}\tAmount: {:>3} satoshis\n\
             Recipients: {:>2}\tAmount: {:>3} satoshis\n",
            self.input.puzzle(),
            self.input.amount(),
            self.outputs[0].puzzle(),
            self.outputs[0].amount()
        )?;
        for output in &self.outputs[1..] {
            write!(
                f,
                "\t\t{:>2}\tAmount: {:>3} satoshis\n",
                output.puzzle(),
                output.amount()
            )?;
        }
        Ok(())
    }
}

pub mod merkle_tree;
pub mod pool;
