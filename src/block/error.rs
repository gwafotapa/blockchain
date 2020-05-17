use std::error;
use std::fmt;

#[derive(Debug)]
pub enum BlockError {
    WrongTransactionCount,
}

impl fmt::Display for BlockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::WrongTransactionCount => {
                write!(f, "Block: number of transactions is not a power of 2")
            }
        }
    }
}

impl error::Error for BlockError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::WrongTransactionCount => None,
        }
    }
}
