use std::error;
use std::fmt;

#[derive(Debug)]
pub enum UtxoPoolError {
    KnownUtxo,
    UnknownUtxo,
    TransactionHasDoubleSpending,
    TransactionHasUnknownUtxo,
    TransactionHasInvalidSignature(secp256k1::Error),
    WrongTransactionCount,
    BlockHasDoubleSpending,
}

impl fmt::Display for UtxoPoolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::KnownUtxo => write!(
                f,
                "Utxo pool: cannot add utxo to the pool that already has it"
            ),
            Self::UnknownUtxo => write!(
                f,
                "Utxo pool: cannot remove utxo from the pool that does not have it"
            ),
            Self::TransactionHasDoubleSpending => {
                write!(f, "Utxo pool: transaction has double-spending")
            }
            Self::TransactionHasUnknownUtxo => write!(f, "Utxo pool: transaction has unknown utxo"),
            Self::TransactionHasInvalidSignature(err) => {
                write!(f, "Utxo pool: ")?;
                err.fmt(f)
            }
            Self::WrongTransactionCount => write!(
                f,
                "Utxo pool: number of transactions in the block is not a power of 2"
            ),
            Self::BlockHasDoubleSpending => write!(f, "Utxo pool: block has double-spending"),
        }
    }
}

impl error::Error for UtxoPoolError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::KnownUtxo => None,
            Self::UnknownUtxo => None,
            Self::TransactionHasDoubleSpending => None,
            Self::TransactionHasUnknownUtxo => None,
            Self::TransactionHasInvalidSignature(err) => err.source(),
            Self::WrongTransactionCount => None,
            Self::BlockHasDoubleSpending => None,
        }
    }
}

impl From<secp256k1::Error> for UtxoPoolError {
    fn from(err: secp256k1::Error) -> Self {
        Self::TransactionHasInvalidSignature(err)
    }
}
