use merkle_cbt::merkle_tree::CBMT;
// use rand::Rng;
use sha2::{Digest, Sha256};
use std::convert::TryInto;
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
    inputs: Vec<Utxo>,
    outputs: Vec<Utxo>,
}

impl Transaction {
    pub fn new(inputs: Vec<Utxo>, outputs: Vec<Utxo>) -> Self {
        Self { inputs, outputs }
    }

    pub fn inputs(&self) -> &[Utxo] {
        &self.inputs
    }

    pub fn outputs(&self) -> &[Utxo] {
        &self.outputs
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.inputs
            .len()
            .to_be_bytes()
            .iter()
            .copied()
            .chain(self.inputs.iter().flat_map(|o| o.serialize()))
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
        let len_inputs = usize::from_be_bytes(bytes[0..8].try_into().unwrap());
        let inputs = bytes[8..]
            .chunks_exact(SIZE_BYTES)
            .take(len_inputs)
            .map(|c| Utxo::from(c))
            .collect();
        let outputs = bytes[8..]
            .chunks_exact(SIZE_BYTES)
            .skip(len_inputs)
            .map(|c| Utxo::from(c))
            .collect();
        Self { inputs, outputs }
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for input in &self.inputs {
            write!(
                f,
                "Sender:    {:>2}\tAmount: {:>3} satoshis\n",
                input.puzzle(),
                input.amount()
            )?;
        }
        for output in &self.outputs {
            write!(
                f,
                "Recipient: {:>2}\tAmount: {:>3} satoshis\n",
                output.puzzle(),
                output.amount()
            )?;
        }
        Ok(())
    }
}

pub mod merkle_tree;
pub mod pool;
