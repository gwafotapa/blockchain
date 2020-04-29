use secp256k1::PublicKey;
use std::convert::TryInto;
use std::fmt;

use crate::common::TX_OUTPUT_BYTES;
use crate::utxo::UtxoData;

#[derive(Clone, Debug)]
pub struct TransactionOutput(UtxoData);

impl TransactionOutput {
    pub fn new(amount: u32, public_key: PublicKey) -> Self {
        Self(UtxoData::new(amount, public_key))
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.0.serialize()
    }

    pub fn deserialize<B>(bytes: B) -> Self
    where
        B: AsRef<[u8]>,
    {
        Self(UtxoData::deserialize(bytes))
    }

    pub fn amount(&self) -> u32 {
        self.0.amount()
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.0.public_key()
    }
}

// impl From<&[u8]> for TransactionOutput {
//     fn from(bytes: &[u8]) -> Self {
//         let amount = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
//         let public_key = PublicKey::from_slice(bytes[4..37].try_into().unwrap()).unwrap();
//         Self { amount, public_key }
//     }
// }

impl fmt::Display for TransactionOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // write!(
        //     f,
        //     "amount: {}\n\
        //      public_key: {}",
        //     self.amount, self.public_key
        // )
        write!(f, "{}", self.0)
    }
}
