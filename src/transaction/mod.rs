use merkle_cbt::merkle_tree::CBMT;
// use rand::Rng;
use sha2::{Digest, Sha256};
// use std::convert::TryInto;
use std::fmt;

use self::merkle_tree::MergeHash;
use crate::common::Hash;
use crate::utxo::Utxo;

pub use pool::TransactionPool;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Transaction {
    input: Utxo,
    output: Utxo,
}

impl From<&[u8]> for Transaction {
    fn from(bytes: &[u8]) -> Self {
        let input = Utxo::from(&bytes[0..12]);
        let output = Utxo::from(&bytes[12..24]);
        Self { input, output }
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Sender:    {}\n\
             Recipient: {}\n\
             Amount:    {} satoshis",
            self.input.puzzle(),
            self.output.puzzle(),
            self.input.amount()
        )
    }
}

impl Transaction {
    pub fn new(input: Utxo, output: Utxo) -> Self {
        Self { input, output }
    }

    // pub fn random(utxo_pool: &UtxoPool) -> Self {
    //     let input = utxo_pool.random();
    //     let output = Utxo::new(input.amount(), utxo_pool.total());
    //     Self { input, output }
    // }

    pub fn input(&self) -> Utxo {
        self.input
    }

    pub fn output(&self) -> Utxo {
        self.output
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.input
            .serialize()
            .iter()
            .chain(self.output.serialize().iter())
            .copied()
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

    // pub fn find(probability: f64, utxo_pool: &UtxoPool) -> Option<Self> {
    //     let mut rng = rand::thread_rng();
    //     match rng.gen_bool(probability) {
    //         false => None,
    //         true => Some(Transaction::random(utxo_pool)),
    //     }
    // }
}

pub mod merkle_tree;
pub mod pool;
