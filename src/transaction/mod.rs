use merkle_cbt::merkle_tree::CBMT;
use secp256k1::{Message as MessageToSign, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::convert::TryInto;
use std::fmt;
use std::hash::{Hash as HashTrait, Hasher};
use std::iter;

use self::merkle_tree::MergeHash;
use crate::constants::{TX_INPUT_BYTES, TX_OUTPUT_BYTES};
use crate::error::transaction::TransactionError;
use crate::utxo::UtxoId;
use crate::Hash;

pub use self::input::TransactionInput;
pub use self::output::TransactionOutput;

#[derive(Clone, Debug)]
pub struct Transaction {
    id: Hash,
    inputs: Vec<TransactionInput>,
    outputs: Vec<TransactionOutput>,
}

// TODO: Check transaction has at least 1 input and 1 output ? (and add a unitary test)
// Also check the sum of the inputs amount equals the sum of the outputs amount.
// And that all inputs share a common public key.
impl Transaction {
    pub fn new(inputs: Vec<TransactionInput>, outputs: Vec<TransactionOutput>) -> Self {
        let mut hasher = Sha256::new();
        let bytes: Vec<u8> = inputs
            .iter()
            .flat_map(|i| i.serialize())
            .chain(outputs.iter().flat_map(|o| o.serialize()))
            .collect();
        hasher.input(bytes);
        let id = hasher.result();
        Self {
            id,
            inputs,
            outputs,
        }
    }

    pub fn sign(
        utxo_ids: Vec<UtxoId>,
        outputs: Vec<TransactionOutput>,
        secret_key: &SecretKey,
    ) -> Self {
        let mut message = Vec::new();
        for utxo_id in &utxo_ids {
            message.extend(utxo_id.serialize());
        }
        for output in &outputs {
            message.extend(output.serialize());
        }
        let mut hasher = Sha256::new();
        hasher.input(message);
        let hash = hasher.result();
        let message = MessageToSign::from_slice(&hash).unwrap();
        let secp = Secp256k1::new();
        let sig = secp.sign(&message, &secret_key);
        let inputs = utxo_ids
            .iter()
            .map(|id| TransactionInput::new(*id, sig))
            .collect();
        Transaction::new(inputs, outputs)
    }

    pub fn serialize(&self) -> Vec<u8> {
        iter::once(b't')
            .chain(self.bytes().to_be_bytes().iter().copied())
            .chain(self.inputs.len().to_be_bytes().iter().copied())
            .chain(self.outputs.len().to_be_bytes().iter().copied())
            .chain(self.inputs.iter().flat_map(|i| i.serialize()))
            .chain(self.outputs.iter().flat_map(|o| o.serialize()))
            .collect()
    }

    pub fn deserialize<B>(bytes: B) -> (Self, usize)
    where
        B: AsRef<[u8]>,
    {
        let bytes = bytes.as_ref();
        let mut i = 1;
        let size = usize::from_be_bytes(bytes[i..i + 8].try_into().unwrap());
        i += 8;
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
        (Self::new(inputs, outputs), size)
    }

    pub fn hash_merkle_root(transactions: &Vec<Self>) -> Hash {
        let hashes = transactions.iter().map(|x| x.id).collect();
        let merkle_tree = CBMT::<Hash, MergeHash>::build_merkle_tree(hashes);
        merkle_tree.root()
    }

    pub fn shares_utxo_with(&self, other: &Transaction) -> bool {
        for self_input in self.inputs() {
            for other_input in other.inputs() {
                if self_input.utxo_id() == other_input.utxo_id() {
                    return true;
                }
            }
        }
        false
    }

    pub fn check_double_spending(&self) -> Result<(), TransactionError> {
        let input_utxos: HashSet<_> = self.inputs().iter().map(|i| *i.utxo_id()).collect();
        if input_utxos.len() == self.inputs().len() {
            Ok(())
        } else {
            Err(TransactionError::DoubleSpending)
        }
    }

    pub fn bytes(&self) -> usize {
        1 + 3 * 8 + self.inputs.len() * TX_INPUT_BYTES + self.outputs.len() * TX_OUTPUT_BYTES
    }

    pub fn id(&self) -> &Hash {
        &self.id
    }

    pub fn inputs(&self) -> &Vec<TransactionInput> {
        &self.inputs
    }

    pub fn outputs(&self) -> &Vec<TransactionOutput> {
        &self.outputs
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
        write!(f, "Transaction {{\n")?;
        for (i, input) in self.inputs().iter().enumerate() {
            write!(
                f,
                "  Input {}:  txid: {:x}  vout: {}\n",
                i,
                input.txid(),
                input.vout(),
            )?;
        }
        for (o, output) in self.outputs().iter().enumerate() {
            write!(
                f,
                "  Output {}:  public_key: {}  amount: {}\n",
                o,
                output.public_key(),
                output.amount(),
            )?;
        }
        write!(f, "}}\n")
    }
}

impl HashTrait for Transaction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

pub mod input;
pub mod merkle_tree;
pub mod output;
