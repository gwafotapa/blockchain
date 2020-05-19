use std::error;
use std::fmt;

#[derive(Debug)]
pub enum BlockchainError {
    KnownBlock,
    OrphanBlock,
    KnownTransactionId,
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
            Self::KnownTransactionId => write!(
                f,
                "Blockchain: block contains a transaction whose id already belongs to the blockchain")
        }
    }
}

impl error::Error for BlockchainError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::KnownBlock => None,
            Self::OrphanBlock => None,
            Self::KnownTransactionId => None,
        }
    }
}
