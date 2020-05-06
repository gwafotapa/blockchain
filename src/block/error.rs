use crate::transaction::error::TransactionError;
use std::error;
use std::fmt;

#[derive(Debug)]
pub enum BlockError {
    WrongTransactionCount,
    InvalidTransaction(TransactionError),
}

impl fmt::Display for BlockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BlockError::WrongTransactionCount => {
                write!(f, "Number of transactions is not a power of 2")
            }
            BlockError::InvalidTransaction(err) => err.fmt(f),
        }
    }
}

impl error::Error for BlockError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            BlockError::WrongTransactionCount => None,
            BlockError::InvalidTransaction(err) => err.source(),
        }
    }
}

// impl From<TransactionError> for BlockError {
//     fn from(err: secp256k1::Error) -> Self {
//         BlockError::InvalidTransaction(err)
//     }
// }
