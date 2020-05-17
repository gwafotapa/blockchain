use std::error;
use std::fmt;

#[derive(Debug)]
pub enum WalletError {
    WrongPublicKey,
    KnownUtxo,
    UnknownUtxo,
}

impl fmt::Display for WalletError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::WrongPublicKey => write!(
                f,
                "Wallet: cannot add utxo to the wallet because public keys do not match"
            ),
            Self::KnownUtxo => write!(
                f,
                "Wallet: cannot add utxo to the wallet that already has it"
            ),
            Self::UnknownUtxo => write!(
                f,
                "Wallet: cannot remove utxo from the wallet that does not have it"
            ),
        }
    }
}

impl error::Error for WalletError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::WrongPublicKey => None,
            Self::KnownUtxo => None,
            Self::UnknownUtxo => None,
        }
    }
}
