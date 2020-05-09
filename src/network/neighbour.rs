use secp256k1::PublicKey;
use std::sync::mpsc::Sender;
use std::sync::Arc;

pub struct Neighbour {
    id: usize,
    public_key: PublicKey,
    sender: Sender<Arc<Vec<u8>>>,
}

impl Neighbour {
    pub fn new(id: usize, public_key: PublicKey, sender: Sender<Arc<Vec<u8>>>) -> Self {
        Self {
            id,
            public_key,
            sender,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    pub fn sender(&self) -> &Sender<Arc<Vec<u8>>> {
        &self.sender
    }
}
