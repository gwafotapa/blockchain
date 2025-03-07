use secp256k1::PublicKey;

pub use self::data::UtxoData;
pub use self::id::UtxoId;
use crate::Hash;
use std::fmt;

#[derive(Clone, Copy, Debug, Hash)]
pub struct Utxo {
    id: UtxoId,
    data: UtxoData,
}

impl Utxo {
    pub fn new(id: UtxoId, data: UtxoData) -> Self {
        Self { id, data }
    }

    pub fn id(&self) -> &UtxoId {
        &self.id
    }

    pub fn data(&self) -> &UtxoData {
        &self.data
    }

    pub fn txid(&self) -> &Hash {
        self.id.txid()
    }

    pub fn vout(&self) -> usize {
        self.id.vout()
    }

    pub fn amount(&self) -> u32 {
        self.data.amount()
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.data.public_key()
    }

    pub fn utxo_id(&self) -> &UtxoId {
        &self.id
    }

    pub fn utxo_data(&self) -> &UtxoData {
        &self.data
    }
}

impl Eq for Utxo {}

impl PartialEq for Utxo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl fmt::Display for Utxo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Utxo {{\n  txid: {:x}\n  vout: {}\n  amount: {}\n  public_key: {}\n}}\n",
            self.txid(),
            self.vout(),
            self.amount(),
            self.public_key()
        )
    }
}

pub mod data;
pub mod id;
