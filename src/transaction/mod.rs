use generic_array::{typenum::U32, GenericArray};
use merkle_cbt::merkle_tree::CBMT;
use rand::{distributions::Alphanumeric, Rng};
use sha2::{Digest, Sha256};
use std::convert::TryInto;

use merkle_tree::MergeHash;

type Hash = GenericArray<u8, U32>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transaction {
    sender: String,
    receiver: String,
    amount: u32,
}

impl From<&[u8]> for Transaction {
    fn from(data: &[u8]) -> Self {
        // TODO: rewrite with iterators ?
        let sender = String::from_utf8(data[0..10].to_vec()).unwrap();
        let receiver = String::from_utf8(data[10..20].to_vec()).unwrap();
        let amount = u32::from_be_bytes(data[20..24].try_into().unwrap());
        Self {
            sender,
            receiver,
            amount,
        }
    }
}

// impl Into<Vec<u8>> for Transaction {
//     fn into(self) -> Vec<u8> {
//         self.sender
//             .as_bytes()
//             .iter()
//             .chain(self.receiver.as_bytes().iter())
//             .chain(self.amount.to_be_bytes().iter())
//             .cloned()
//             .collect()
//     }
// }

impl Transaction {
    pub fn new<S>(sender: S, receiver: S, amount: u32) -> Self
    where
        S: Into<String>,
    {
        Self {
            sender: sender.into(),
            receiver: receiver.into(),
            amount,
        }
    }

    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let sender = rng.sample_iter(&Alphanumeric).take(10).collect::<String>();
        let receiver = rng.sample_iter(&Alphanumeric).take(10).collect::<String>();
        let amount = rng.gen::<u32>();
        Self {
            sender,
            receiver,
            amount,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        std::iter::once(b't')
            .chain(self.sender.bytes())
            .chain(self.receiver.bytes())
            .chain(self.amount.to_be_bytes().iter().copied())
            .collect()
    }

    // pub fn from_bytes(data: &[u8]) -> Self {
    //     let sender = String::from_utf8(data[0..10].to_vec()).unwrap();
    //     let receiver = String::from_utf8(data[10..20].to_vec()).unwrap();
    //     let amount = u32::from_be_bytes(data[20..24].try_into().unwrap());
    //     Self {
    //         sender,
    //         receiver,
    //         amount,
    //     }
    // }

    pub fn hash(&self) -> Hash {
        let mut hasher = Sha256::new();
        hasher.input(self.to_bytes());
        hasher.result()
    }

    pub fn hash_merkle_root(transactions: &Vec<Self>) -> Hash {
        let hashes = transactions.iter().map(|x| x.hash()).collect();
        let merkle_tree = CBMT::<Hash, MergeHash>::build_merkle_tree(hashes);
        merkle_tree.root()
    }

    pub fn find(probability: f64) -> Option<Self> {
        let mut rng = rand::thread_rng();
        match rng.gen_bool(probability) {
            false => None,
            true => Some(Transaction::random()),
        }
    }
}

pub mod merkle_tree;
