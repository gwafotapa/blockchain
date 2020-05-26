use std::error;
use std::fmt;

use crate::Hash as TransactionId;

#[derive(Debug)]
pub enum TransactionPoolError {
    KnownTransaction,
    UnknownTransaction,
    UnknownUtxo(TransactionId),
}

impl fmt::Display for TransactionPoolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::KnownTransaction => write!(
                f,
                "Transaction pool: cannot add transaction to the pool that already has it"
            ),
            Self::UnknownTransaction => write!(
                f,
                "Transaction pool: cannot remove transaction from the pool that does not have it"
            ),
            Self::UnknownUtxo(txid) => write!(
                f,
                "Transaction pool: transaction {:x} has unknown utxo",
                txid
            ),
        }
    }
}

impl error::Error for TransactionPoolError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::KnownTransaction => None,
            Self::UnknownTransaction => None,
            Self::UnknownUtxo(_) => None,
        }
    }
}
