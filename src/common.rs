use generic_array::{typenum::U32, GenericArray};
use std::borrow::Cow;

// use crate::block::Block;
use crate::transaction::Transaction;

pub type Hash = GenericArray<u8, U32>;

pub const NODES: usize = 4;
pub const PROBABILITY_SPEND: f64 = 1.0 / 1000000.0;
const SHUT_DOWN: &[u8] = b"Shut down";

pub enum Message<'a> {
    Transaction(Cow<'a, Transaction>),
    // Block(Block),
    ShutDown,
}

impl<'a, T> From<T> for Message<'a>
where
    T: AsRef<[u8]>,
{
    fn from(bytes: T) -> Self {
        let bytes = bytes.as_ref();
        if &bytes[..] == SHUT_DOWN {
            return Message::ShutDown;
        }
        match bytes[0] {
            b't' => Message::Transaction(Cow::Owned(Transaction::from(&bytes[1..]))),
            _ => panic!("Unexpected message"),
        }
    }
}

impl<'a> Message<'a> {
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            Message::Transaction(transaction) => transaction.serialize(),
            Message::ShutDown => SHUT_DOWN.to_vec(),
        }
    }
}
