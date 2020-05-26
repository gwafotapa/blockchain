use std::error;
use std::fmt;

#[derive(Debug)]
pub enum BlockError {
    WrongTransactionCount,
    InvalidExponentOfTarget(u8),
    DoubleSpending,
}

impl fmt::Display for BlockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::WrongTransactionCount => {
                write!(f, "Block: number of transactions is not a power of 2")
            }
            Self::InvalidExponentOfTarget(exponent) => write!(
                f,
                "Block: exponent {} of the target is not between 3 and 32 (included)",
                exponent
            ),
            Self::DoubleSpending => write!(f, "Block: double spending detected"),
        }
    }
}

impl error::Error for BlockError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::WrongTransactionCount => None,
            Self::InvalidExponentOfTarget(_) => None,
            Self::DoubleSpending => None,
        }
    }
}
