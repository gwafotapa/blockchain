use std::error;
use std::fmt;

#[derive(Debug)]
pub enum BlockchainError {
    KnownBlock,
    OrphanBlock,
}

impl fmt::Display for BlockchainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BlockchainError::KnownBlock => write!(f, "Block already belongs to the blockchain"),
            BlockchainError::OrphanBlock => write!(f, "Block has no parent in the blockchain"),
        }
    }
}

impl error::Error for BlockchainError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            BlockchainError::KnownBlock => None,
            BlockchainError::OrphanBlock => None,
        }
    }
}
