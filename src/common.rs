use generic_array::{typenum::U32, GenericArray};
use std::borrow::Cow;

use crate::transaction::Transaction;

pub type Hash = GenericArray<u8, U32>;

pub const GENESIS_BLOCK_HASH_PREV_BLOCK: [u8; 32] = [0u8; 32];
pub const NODES: usize = 4;
pub const SIGNATURE_BYTES: usize = 64;
pub const SPEND_PROBA: f64 = 1.0 / 1000000.0;
pub const TX_INPUT_BYTES: usize = UTXO_ID_BYTES + SIGNATURE_BYTES;
pub const TX_OUTPUT_BYTES: usize = UTXO_DATA_BYTES;
pub const UTXO_AMOUNT_INIT: u32 = 10;
pub const UTXO_DATA_BYTES: usize = 4 + 33;
pub const UTXO_HASH_INIT: [u8; 32] = [0u8; 32];
pub const UTXO_ID_BYTES: usize = 32 + 8;

const SHUT_DOWN: &[u8] = b"Shut down";

pub enum Message<'a> {
    Transaction(Cow<'a, Transaction>),
    ShutDown,
}

impl<'a, B> From<B> for Message<'a>
where
    B: AsRef<[u8]>,
{
    fn from(bytes: B) -> Self {
        let bytes = bytes.as_ref();
        if &bytes[..] == SHUT_DOWN {
            return Message::ShutDown;
        }
        match bytes[0] {
            b't' => Message::Transaction(Cow::Owned(Transaction::from(&bytes[..]))),
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

    pub fn deserialize<T>(bytes: T) -> Self
    where
        T: AsRef<[u8]>,
    {
        Self::from(bytes)
    }
}
