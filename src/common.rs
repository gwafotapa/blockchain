use generic_array::{typenum::U32, GenericArray};
use std::borrow::Borrow;

// use crate::block::Block;
use crate::transaction::Transaction;

type Hash = GenericArray<u8, U32>;

pub const NODES: usize = 10;
pub const PROBABILITY_SPEND: f64 = 1.0 / 1000000.0;

pub enum Data {
    Transaction(Transaction),
    // Block(Block),
}

impl<'a, T> From<T> for Data
where
    T: AsRef<[u8]>,
{
    fn from(bytes: T) -> Self {
        let bytes = bytes.as_ref();
        match bytes[0] {
            b't' => Data::Transaction(Transaction::from(&bytes[1..])),
            _ => panic!("Unexpected data"),
        }
    }
}
