use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct InvalidTransaction;

impl fmt::Display for InvalidTransaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid transaction")
    }
}

impl Error for InvalidTransaction {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
