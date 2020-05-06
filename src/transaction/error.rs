use std::error;
use std::fmt;

#[derive(Debug)]
pub enum TransactionError {
    UnknownUtxo,
    InvalidSignature(secp256k1::Error),
}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TransactionError::UnknownUtxo => write!(f, "Unknown utxo"),
            TransactionError::InvalidSignature(err) => err.fmt(f),
        }
    }
}

impl error::Error for TransactionError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            TransactionError::UnknownUtxo => None,
            TransactionError::InvalidSignature(err) => err.source(),
        }
    }
}

impl From<secp256k1::Error> for TransactionError {
    fn from(err: secp256k1::Error) -> Self {
        TransactionError::InvalidSignature(err)
    }
}
