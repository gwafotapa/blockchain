use hex::ToHex;
use std::error;
use std::fmt;

use crate::common::Hash;

#[derive(Debug)]
pub enum TransactionError {
    UnknownUtxo,
    InvalidSignature(secp256k1::Error),
    PoolSpentUtxo(Hash),
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
        }
    }
}

impl error::Error for TransactionError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            TransactionError::UnknownUtxo => None,
            TransactionError::InvalidSignature(err) => err.source(),
            TransactionError::PoolSpentUtxo(_) => None,
        }
    }
}

impl From<secp256k1::Error> for TransactionError {
    fn from(err: secp256k1::Error) -> Self {
        TransactionError::InvalidSignature(err)
    }
}
