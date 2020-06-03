use std::error;
use std::fmt;

#[derive(Debug)]
pub enum TransactionError {
    NoInputs,
    NoOutputs,
    DoubleSpending,
    WrongBalance,
}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::NoInputs => write!(f, "Transaction: no inputs"),
            Self::NoOutputs => write!(f, "Transaction: no outputs"),
            Self::DoubleSpending => write!(f, "Transaction: double spending"),
            Self::WrongBalance => write!(f, "Transaction: input and output amounts differ"),
        }
    }
}

impl error::Error for TransactionError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::NoInputs => None,
            Self::NoOutputs => None,
            Self::DoubleSpending => None,
            Self::WrongBalance => None,
        }
    }
}
