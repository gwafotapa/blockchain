use generic_array::{typenum::U32, GenericArray};
use merkle_cbt::merkle_tree::{Merge, CBMT};
use rand::{distributions::Alphanumeric, Rng};
use sha2::{Digest, Sha256};

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
        bytes.extend_from_slice(self.sender.as_bytes());
        bytes.extend_from_slice(self.receiver.as_bytes());
        bytes.extend_from_slice(&self.amount.to_be_bytes());
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

pub struct MergeHash {}

impl Merge for MergeHash {
    type Item = Hash;

    fn merge(left: &Self::Item, right: &Self::Item) -> Self::Item {
        let mut concat = Vec::from(left.as_slice());
        concat.extend_from_slice(right.as_slice());
        let mut hasher = Sha256::new();
        hasher.input(concat);
        hasher.result()
    }
}
