use generic_array::{typenum::U32, GenericArray};
use std::borrow::Cow;

// use crate::block::Block;
use crate::transaction::Transaction;

pub type Hash = GenericArray<u8, U32>;

pub const NODES: usize = 4;
pub const PROBABILITY_SPEND: f64 = 1.0 / 1000000.0;
const SHUT_DOWN: &[u8] = b"Shut down";
pub const INPUT_SIZE_BYTES: usize = 32 + 8;
pub const OUTPUT_SIZE_BYTES: usize = 4 + 8;

pub enum Message<'a> {
    Transaction(Cow<'a, Transaction>),
    // Block(Block),
    ShutDown,
}

// impl<'a, T> From<T> for Message<'a>
// where
//     T: AsRef<[u8]>,
// {
//     fn from(bytes: T) -> Self {
//         let bytes = bytes.as_ref();
//         if &bytes[..] == SHUT_DOWN {
//             return Message::ShutDown;
//         }
//         match bytes[0] {
//             b't' => Message::Transaction(Cow::Owned(Transaction::from(&bytes[1..]))),
//             _ => panic!("Unexpected message"),
//         }
//     }
// }

impl<'a> Message<'a> {
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            Message::Transaction(transaction) => transaction.serialize(),
            Message::ShutDown => SHUT_DOWN.to_vec(),
        }
    }

    // TODO: use from trait ?
    pub fn from<T>(bytes: T) -> Vec<Self>
    where
        T: AsRef<[u8]>,
    {
        let mut bytes = bytes.as_ref();
        let mut vec = Vec::new();
        while !bytes.is_empty() {
            if &bytes[..] == SHUT_DOWN {
                vec.push(Message::ShutDown);
                let len = SHUT_DOWN.len();
                bytes = &bytes[len..];
            } else {
                match bytes[0] {
                    b't' => {
                        let transaction = Transaction::from(&bytes[..]);
                        let len = 1
                            + 8
                            + transaction.inputs().len() * INPUT_SIZE_BYTES
                            + 8
                            + transaction.outputs().len() * OUTPUT_SIZE_BYTES;
                        bytes = &bytes[len..];
                        let message = Message::Transaction(Cow::Owned(transaction));
                        vec.push(message);
                    }
                    _ => panic!("Unexpected message"),
                }
            }
        }
        vec
    }
}
