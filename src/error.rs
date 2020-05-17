use std::error;
use std::fmt;

use crate::block::error::BlockError;
use crate::blockchain::error::BlockchainError;
use crate::transaction_pool::error::TransactionPoolError;
use crate::utxo_pool::error::UtxoPoolError;
use crate::wallet::error::WalletError;

#[derive(Debug)]
pub enum Error {
    BlockError(BlockError),
    BlockchainError(BlockchainError),
    TransactionPoolError(TransactionPoolError),
    UtxoPoolError(UtxoPoolError),
    WalletError(WalletError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BlockError(err) => err.fmt(f),
            Self::BlockchainError(err) => err.fmt(f),
            Self::TransactionPoolError(err) => err.fmt(f),
            Self::UtxoPoolError(err) => err.fmt(f),
            Self::WalletError(err) => err.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::BlockError(err) => err.source(),
            Self::BlockchainError(err) => err.source(),
            Self::TransactionPoolError(err) => err.source(),
            Self::UtxoPoolError(err) => err.source(),
            Self::WalletError(err) => err.source(),
        }
    }
}

impl From<BlockError> for Error {
    fn from(err: BlockError) -> Self {
        Self::BlockError(err)
    }
}

impl From<BlockchainError> for Error {
    fn from(err: BlockchainError) -> Self {
        Self::BlockchainError(err)
    }
}

impl From<TransactionPoolError> for Error {
    fn from(err: TransactionPoolError) -> Self {
        Self::TransactionPoolError(err)
    }
}

impl From<UtxoPoolError> for Error {
    fn from(err: UtxoPoolError) -> Self {
        Self::UtxoPoolError(err)
    }
}

impl From<WalletError> for Error {
    fn from(err: WalletError) -> Self {
        Self::WalletError(err)
    }
}
