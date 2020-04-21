use generic_array::{typenum::U32, GenericArray};

// use crate::block::Block;
use crate::transaction::Transaction;

pub type Hash = GenericArray<u8, U32>;

pub const NODES: usize = 4;
pub const PROBABILITY_SPEND: f64 = 1.0 / 1000000.0;

pub enum Data {
    Transaction(Transaction),
    // Block(Block),
    ShutDown,
}

impl<T> From<T> for Data
where
    T: AsRef<[u8]>,
{
    fn from(bytes: T) -> Self {
        let bytes = bytes.as_ref();
        if &bytes[..] == b"Shut down" {
            return Data::ShutDown;
        }
        match bytes[0] {
            b't' => Data::Transaction(Transaction::from(&bytes[1..])),
            _ => panic!("Unexpected data"),
        }
    }
}

impl Data {
    pub fn serialize(&self) -> Vec<u8> {
        match *self {
            Data::Transaction(transaction) => transaction.serialize(),
            Data::ShutDown => b"Shut down".to_vec(),
        }
    }
}
