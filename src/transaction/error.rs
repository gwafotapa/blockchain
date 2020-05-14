use hex::ToHex;
use std::error;
use std::fmt;

use crate::Hash;

#[derive(Debug)]
pub enum TransactionError {
    UnknownUtxo,
    InvalidSignature(secp256k1::Error),
    PoolSpentUtxo(Hash),
    PoolHasTransaction,
    UnknownTransaction,
    PoolHasUtxo,
    DuplicateUtxo,
    WrongPublicKey,
    WalletHasUtxo,
}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TransactionError::UnknownUtxo => write!(f, "Unknown utxo"),
            TransactionError::InvalidSignature(err) => err.fmt(f),
            TransactionError::PoolSpentUtxo(txid) => {
                write!(f, "This utxo is already spent by pool transaction ")?;
                txid.write_hex(f)
            }
            TransactionError::PoolHasTransaction => write!(
                f,
                "Cannot add transaction because it is already in the pool"
            ),
            TransactionError::UnknownTransaction => {
                write!(f, "Cannot remove transaction because it is not in the pool")
            }
            TransactionError::PoolHasUtxo => {
                write!(f, "Cannot add utxo because it is already in the pool")
            }
            TransactionError::DuplicateUtxo => {
                write!(f, "Transaction inputs contain two copies of the same utxo")
            }
            TransactionError::WrongPublicKey => {
                write!(f, "Cannot add utxo to the wallet: public keys differ")
            }
            TransactionError::WalletHasUtxo => {
                write!(f, "Cannot add utxo because the wallet already has it")
            }
        }
    }
}

impl error::Error for TransactionError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            TransactionError::UnknownUtxo => None,
            TransactionError::InvalidSignature(err) => err.source(),
            TransactionError::PoolSpentUtxo(_) => None,
            TransactionError::PoolHasTransaction => None,
            TransactionError::UnknownTransaction => None,
            TransactionError::PoolHasUtxo => None,
            TransactionError::DuplicateUtxo => None,
            TransactionError::WrongPublicKey => None,
            TransactionError::WalletHasUtxo => None,
        }
    }
}

impl From<secp256k1::Error> for TransactionError {
    fn from(err: secp256k1::Error) -> Self {
        TransactionError::InvalidSignature(err)
    }
}
