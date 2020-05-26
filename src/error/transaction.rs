use std::error;
use std::fmt;

#[derive(Debug)]
pub enum TransactionError {
    DoubleSpending,
}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::DoubleSpending => write!(f, "Transaction: double spending"),
        }
    }
}

impl error::Error for TransactionError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::DoubleSpending => None,
        }
    }
}
