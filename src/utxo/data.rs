use secp256k1::PublicKey;
use std::convert::TryInto;
use std::fmt;

use crate::common::UTXO_DATA_BYTES;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UtxoData {
    amount: u32,
    public_key: PublicKey,
}

impl UtxoData {
    pub fn new(amount: u32, public_key: PublicKey) -> Self {
        Self { amount, public_key }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(UTXO_DATA_BYTES);
        bytes.extend(&self.amount.to_be_bytes());
        bytes.extend(self.public_key.serialize().iter());
        bytes
    }

    pub fn deserialize<B>(bytes: B) -> Self
    where
        B: AsRef<[u8]>,
    {
        Self::from(bytes)
    }

    pub fn amount(&self) -> u32 {
        self.amount
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }
}

impl<B> From<B> for UtxoData
where
    B: AsRef<[u8]>,
{
    fn from(bytes: B) -> Self {
        let bytes = bytes.as_ref();
        let amount = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
        let public_key = PublicKey::from_slice(bytes[4..37].try_into().unwrap()).unwrap();
        Self { amount, public_key }
    }
}

impl fmt::Display for UtxoData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "amount: {:>7}\t\tpublic_key: {}",
            self.amount, self.public_key
        )
    }
}
