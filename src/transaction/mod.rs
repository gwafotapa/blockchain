use merkle_cbt::merkle_tree::CBMT;
use sha2::{Digest, Sha256};
use std::convert::TryInto;
use std::fmt;
use std::hash::{Hash as HashTrait, Hasher};
use std::iter;

use self::merkle_tree::MergeHash;
use crate::common::{Hash, TX_INPUT_BYTES, TX_OUTPUT_BYTES};

pub use self::error::InvalidTransaction;
pub use self::input::TransactionInput;
pub use self::output::TransactionOutput;
pub use self::pool::TransactionPool;

#[derive(Clone, Debug)]
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
            .chain(self.outputs.len().to_be_bytes().iter().copied())
            .chain(self.inputs.iter().flat_map(|i| i.serialize()))
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

    pub fn id(&self) -> &Hash {
        &self.id
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
        let mut i = 1;
        let inputs_len = usize::from_be_bytes(bytes[i..i + 8].try_into().unwrap());
        i += 8;
        let outputs_len = usize::from_be_bytes(bytes[i..i + 8].try_into().unwrap());
        i += 8;
        let inputs = bytes[i..]
            .chunks_exact(TX_INPUT_BYTES)
            .take(inputs_len)
            .map(|c| TransactionInput::deserialize(c))
            .collect();
        i += inputs_len * TX_INPUT_BYTES;
        let outputs = bytes[i..]
            .chunks_exact(TX_OUTPUT_BYTES)
            .take(outputs_len)
            .map(|c| TransactionOutput::deserialize(c))
            .collect();
        Self::new(inputs, outputs)
    }
}

impl Eq for Transaction {}

impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, input) in self.inputs().iter().enumerate() {
            write!(f, "Input {}:  {}\n", i, input)?;
        }
        for (o, output) in self.outputs().iter().enumerate() {
            write!(f, "Output {}:  {}\n", o, output)?;
        }
        Ok(())
    }
}

impl HashTrait for Transaction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

pub mod error;
pub mod input;
pub mod merkle_tree;
pub mod output;
pub mod pool;
