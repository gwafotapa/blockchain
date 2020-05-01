use std::error;
use std::fmt;

#[derive(Debug)]
pub enum InvalidTransaction {
    UnknownUtxo,
    InvalidSignature(secp256k1::Error),
}

impl fmt::Display for InvalidTransaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InvalidTransaction::UnknownUtxo => write!(f, "Unknown utxo"),
            InvalidTransaction::InvalidSignature(err) => err.fmt(f),
        }
    }
}

impl error::Error for InvalidTransaction {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            InvalidTransaction::UnknownUtxo => None,
            InvalidTransaction::InvalidSignature(err) => err.source(),
        }
    }
}

impl From<secp256k1::Error> for InvalidTransaction {
    fn from(err: secp256k1::Error) -> Self {
        InvalidTransaction::InvalidSignature(err)
    }
}
