use merkle_cbt::merkle_tree::CBMT;
use sha2::{Digest, Sha256};
use std::convert::TryInto;
use std::fmt;
use std::iter;

use self::input::TransactionInput;
use self::merkle_tree::MergeHash;
use self::output::TransactionOutput;
use crate::common::Hash;

const INPUT_SIZE_BYTES: usize = 32 + 8;
const OUTPUT_SIZE_BYTES: usize = 12;

#[derive(Clone, Debug, Eq)]
pub struct Transaction {
    id: Hash,
    inputs: Vec<TransactionInput>,
    outputs: Vec<TransactionOutput>,
}

impl Transaction {
    pub fn new(inputs: Vec<TransactionInput>, outputs: Vec<TransactionOutput>) -> Self {
        let mut hasher = Sha256::new();
        let bytes = inputs
            .iter()
            .flat_map(|i| i.serialize())
            .chain(outputs.iter().flat_map(|o| o.serialize()))
            .collect::<Vec<_>>();
        hasher.input(bytes);
        let id = hasher.result();
        Self {
            id,
            inputs,
            outputs,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        iter::once(b't')
            .chain(self.inputs.len().to_be_bytes().iter().copied())
            .chain(self.inputs.iter().flat_map(|i| i.serialize()))
            .chain(self.outputs.len().to_be_bytes().iter().copied())
            .chain(self.outputs.iter().flat_map(|o| o.serialize()))
            .collect()
    }

    pub fn deserialize<B>(bytes: B) -> Self
    where
        B: AsRef<[u8]>,
    {
        Self::from(bytes.as_ref())
    }

    pub fn hash_merkle_root(transactions: &Vec<Self>) -> Hash {
        let hashes = transactions.iter().map(|x| x.id).collect();
        let merkle_tree = CBMT::<Hash, MergeHash>::build_merkle_tree(hashes);
        merkle_tree.root()
    }

    pub fn id(&self) -> Hash {
        self.id
    }

    pub fn inputs(&self) -> &[TransactionInput] {
        &self.inputs
    }

    pub fn outputs(&self) -> &[TransactionOutput] {
        &self.outputs
    }
}

impl From<&[u8]> for Transaction {
    fn from(bytes: &[u8]) -> Self {
        let inputs_len = usize::from_be_bytes(bytes[8..16].try_into().unwrap());
        let inputs = bytes[16..]
            .chunks_exact(INPUT_SIZE_BYTES)
            .take(inputs_len)
            .map(|c| TransactionInput::from(c))
            .collect();
        let i = 16 + inputs_len * INPUT_SIZE_BYTES;
        let outputs_len = usize::from_be_bytes(bytes[i..i + 8].try_into().unwrap());
        let outputs = bytes[i + 8..]
            .chunks_exact(OUTPUT_SIZE_BYTES)
            .take(outputs_len)
            .map(|c| TransactionOutput::from(c))
            .collect();
        Self::new(inputs, outputs)
    }
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for input in &self.inputs {
            write!(f, "{:?}\n", input)?;
        }
        for output in &self.outputs {
            write!(f, "{:?}\n", output)?;
        }
        Ok(())
    }
}

pub mod error;
pub mod input;
pub mod merkle_tree;
pub mod output;
pub mod pool;
