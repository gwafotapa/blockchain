use secp256k1::PublicKey;
use std::fmt;

use crate::utxo::UtxoData;

#[derive(Clone, Copy, Debug)]
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

    pub fn utxo_data(&self) -> &UtxoData {
        &self.0
    }

    pub fn amount(&self) -> u32 {
        self.0.amount()
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.0.public_key()
    }
}

impl From<UtxoData> for TransactionOutput {
    fn from(utxo_data: UtxoData) -> Self {
        Self(utxo_data)
    }
}

impl fmt::Display for TransactionOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Transaction output {{\n  amount: {}\n  pulic_key: {}\n}}",
            self.amount(),
            self.public_key()
        )
    }
}
