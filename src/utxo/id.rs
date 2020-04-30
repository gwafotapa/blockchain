use hex::ToHex;
use std::convert::TryInto;
use std::fmt;

use crate::common::{Hash, UTXO_ID_BYTES};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct UtxoId {
    txid: Hash,
    vout: usize,
}

impl UtxoId {
    pub fn new(txid: Hash, vout: usize) -> Self {
        Self { txid, vout }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(UTXO_ID_BYTES);
        bytes.extend(self.txid.as_slice());
        bytes.extend(&self.vout.to_be_bytes());
        bytes
    }

    pub fn deserialize<B>(bytes: B) -> Self
    where
        B: AsRef<[u8]>,
    {
        Self::from(bytes.as_ref())
    }

    pub fn txid(&self) -> &Hash {
        &self.txid
    }

    pub fn vout(&self) -> usize {
        self.vout
    }
}

impl From<&[u8]> for UtxoId {
    fn from(bytes: &[u8]) -> Self {
        let txid = *Hash::from_slice(&bytes[..32]);
        let vout = usize::from_be_bytes(bytes[32..40].try_into().unwrap());
        Self { txid, vout }
    }
}

impl fmt::Display for UtxoId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "txid: ")?;
        self.txid.write_hex(f)?;
        write!(f, "\tvout: {}", self.vout)
    }
}
