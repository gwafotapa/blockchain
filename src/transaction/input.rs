use std::convert::TryInto;

use crate::common::{Hash, INPUT_SIZE_BYTES};
use crate::utxo::UtxoId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionInput {
    utxo_id: UtxoId,
}

impl TransactionInput {
    pub fn new(utxo_id: UtxoId) -> Self {
        Self { utxo_id }
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.utxo_id.serialize()
    }

    pub fn deserialize<B>(bytes: B) -> Self
    where
        B: AsRef<[u8]>,
    {
        Self::from(bytes.as_ref())
    }

    pub fn utxo_id(&self) -> &UtxoId {
        &self.utxo_id
    }

    pub fn txid(&self) -> &Hash {
        &self.utxo_id.txid()
    }

    pub fn vout(&self) -> usize {
        self.utxo_id.vout()
    }
}

impl From<&[u8]> for TransactionInput {
    fn from(bytes: &[u8]) -> Self {
        let utxo_id = UtxoId::deserialize(bytes);
        Self { utxo_id }
    }
}
