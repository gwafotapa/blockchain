use secp256k1::Signature;
use std::fmt;

use crate::common::{Hash, TX_INPUT_BYTES, UTXO_ID_BYTES};
use crate::utxo::UtxoId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionInput {
    utxo_id: UtxoId,
    sig: Signature,
}

impl TransactionInput {
    pub fn new(utxo_id: UtxoId, sig: Signature) -> Self {
        Self { utxo_id, sig }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(TX_INPUT_BYTES);
        bytes.extend(self.utxo_id.serialize());
        bytes.extend(self.sig.serialize_compact().iter());
        bytes
    }

    pub fn deserialize<B>(bytes: B) -> Self
    where
        B: AsRef<[u8]>,
    {
        Self::from(bytes)
    }

    pub fn utxo_id(&self) -> &UtxoId {
        &self.utxo_id
    }

    pub fn sig(&self) -> &Signature {
        &self.sig
    }

    pub fn txid(&self) -> Hash {
        self.utxo_id.txid()
    }

    pub fn vout(&self) -> usize {
        self.utxo_id.vout()
    }
}

impl<B> From<B> for TransactionInput
where
    B: AsRef<[u8]>,
{
    fn from(bytes: B) -> Self {
        let bytes = bytes.as_ref();
        let utxo_id = UtxoId::deserialize(bytes);
        let sig = Signature::from_compact(&bytes[UTXO_ID_BYTES..]).unwrap();
        Self { utxo_id, sig }
    }
}

impl fmt::Display for TransactionInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\n\t      sig: {}", self.utxo_id, self.sig)
    }
}
