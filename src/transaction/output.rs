use secp256k1::PublicKey;
use std::convert::TryInto;

use crate::common::OUTPUT_SIZE_BYTES;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TransactionOutput {
    amount: u32,
    public_key: PublicKey,
}

impl TransactionOutput {
    pub fn new(amount: u32, public_key: PublicKey) -> Self {
        Self { amount, public_key }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(OUTPUT_SIZE_BYTES);
        bytes.extend(&self.amount.to_be_bytes());
        bytes.extend(self.public_key.serialize().iter());
        bytes
    }

    pub fn deserialize<B>(bytes: B) -> Self
    where
        B: AsRef<[u8]>,
    {
        Self::from(bytes.as_ref())
    }

    pub fn amount(&self) -> u32 {
        self.amount
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }
}

impl From<&[u8]> for TransactionOutput {
    fn from(bytes: &[u8]) -> Self {
        let amount = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
        let public_key = PublicKey::from_slice(bytes[4..37].try_into().unwrap()).unwrap();
        Self { amount, public_key }
    }
}
