use std::borrow::Cow;

use crate::block::Block;
use crate::transaction::Transaction;

const SHUT_DOWN: &[u8] = b"Shut down";

#[derive(Eq, PartialEq)]
pub enum Message<'a> {
    Transaction(Cow<'a, Transaction>),
    Block(Cow<'a, Block>),
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
            b't' => Message::Transaction(Cow::Owned(Transaction::deserialize(&bytes[..]).0)),
            b'b' => Message::Block(Cow::Owned(Block::deserialize(&bytes[..]))),
            _ => panic!("Unexpected message"),
        }
    }
}

impl<'a> Message<'a> {
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            Message::Transaction(transaction) => transaction.serialize(),
            Message::Block(block) => block.serialize(),
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
