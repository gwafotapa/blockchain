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
            Self::KnownBlock => write!(
                f,
                "Blockchain: cannot add block to the blockchain that already has it"
            ),
            Self::OrphanBlock => write!(
                f,
                "Blockchain: cannot add block to the blockchain that does not have the block's parent"
            ),
        }
    }
}

impl error::Error for BlockchainError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::KnownBlock => None,
            Self::OrphanBlock => None,
        }
    }
}
