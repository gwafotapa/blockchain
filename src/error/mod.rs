use std::error;
use std::fmt;

use self::block::BlockError;
use self::blockchain::BlockchainError;
use self::transaction::TransactionError;
use self::transaction_pool::TransactionPoolError;
use self::utxo_pool::UtxoPoolError;
use self::wallet::WalletError;

#[derive(Debug)]
pub enum Error {
    Block(BlockError),
    Blockchain(BlockchainError),
    Transaction(TransactionError),
    TransactionPool(TransactionPoolError),
    UtxoPool(UtxoPoolError),
    Wallet(WalletError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Block(err) => err.fmt(f),
            Self::Blockchain(err) => err.fmt(f),
            Self::Transaction(err) => err.fmt(f),
            Self::TransactionPool(err) => err.fmt(f),
            Self::UtxoPool(err) => err.fmt(f),
            Self::Wallet(err) => err.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Block(err) => err.source(),
            Self::Blockchain(err) => err.source(),
            Self::Transaction(err) => err.source(),
            Self::TransactionPool(err) => err.source(),
            Self::UtxoPool(err) => err.source(),
            Self::Wallet(err) => err.source(),
        }
    }
}

impl From<BlockError> for Error {
    fn from(err: BlockError) -> Self {
        Self::Block(err)
    }
}

impl From<BlockchainError> for Error {
    fn from(err: BlockchainError) -> Self {
        Self::Blockchain(err)
    }
}

impl From<TransactionError> for Error {
    fn from(err: TransactionError) -> Self {
        Self::Transaction(err)
    }
}

impl From<TransactionPoolError> for Error {
    fn from(err: TransactionPoolError) -> Self {
        Self::TransactionPool(err)
    }
}

impl From<UtxoPoolError> for Error {
    fn from(err: UtxoPoolError) -> Self {
        Self::UtxoPool(err)
    }
}

impl From<WalletError> for Error {
    fn from(err: WalletError) -> Self {
        Self::Wallet(err)
    }
}

pub mod block;
pub mod blockchain;
pub mod transaction;
pub mod transaction_pool;
pub mod utxo_pool;
pub mod wallet;
