use std::convert::TryInto;

use crate::common::{Hash, INPUT_SIZE_BYTES};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TransactionInput {
    txid: Hash,
    vout: usize,
}

impl TransactionInput {
    pub fn new(txid: Hash, vout: usize) -> Self {
        Self { txid, vout }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(INPUT_SIZE_BYTES);
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

    pub fn txid(&self) -> Hash {
        self.txid
    }

    pub fn vout(&self) -> usize {
        self.vout
    }

    // pub fn utxo(&self, tx_pool: &TransactionPool) -> Option<Utxo> {
    //     if let Some(transaction) = tx_pool.transactions().iter().find(|&tx| tx.id == self.txid) {
    //         Some(transaction.outputs()[self.vout])
    //     } else {
    //         None
    //     }
    // }
}

impl From<&[u8]> for TransactionInput {
    fn from(bytes: &[u8]) -> Self {
        let txid = *Hash::from_slice(&bytes[..32]);
        let vout = usize::from_be_bytes(bytes[32..40].try_into().unwrap());
        Self { txid, vout }
    }
}
