use secp256k1::PublicKey;

use crate::utxo::UtxoData;

// TODO: Is the wrapper really necessary ? Should the field be public ?
#[derive(Clone, Debug)]
pub struct TransactionOutput(pub UtxoData);

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

// impl fmt::Display for TransactionOutput {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         self.0.fmt(f)
//     }
// }
