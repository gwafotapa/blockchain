use generic_array::{typenum::U32, GenericArray};
use merkle_cbt::merkle_tree::CBMT;
use rand::{distributions::Alphanumeric, Rng};
use sha2::{Digest, Sha256};

use merkle_tree::MergeHash;

type Hash = GenericArray<u8, U32>;

pub struct Transaction {
    sender: String,
    receiver: String,
    amount: u32,
}

impl Transaction {
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
        let mut bytes = Vec::new();
        bytes.extend(self.sender.bytes());
        bytes.extend(self.receiver.bytes());
        bytes.extend(&self.amount.to_be_bytes());
        bytes
    }

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
}

pub mod merkle_tree;
