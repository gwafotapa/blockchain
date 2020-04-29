use hex::ToHex;
use secp256k1::PublicKey;
use std::convert::TryInto;
use std::fmt;

use crate::common::{Hash, UTXO_DATA_BYTES, UTXO_ID_BYTES};

pub use self::pool::UtxoPool;

#[derive(Clone, Debug)]
pub struct Utxo {
    id: UtxoId,
    data: UtxoData,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct UtxoId {
    txid: Hash,
    vout: usize,
}

#[derive(Clone, Debug)]
pub struct UtxoData {
    amount: u32,
    public_key: PublicKey,
}

impl Utxo {
    pub fn new(id: UtxoId, data: UtxoData) -> Self {
        Self { id, data }
    }
    // pub fn new(txid: Hash, vout: usize, amount: u32, public_key: PublicKey) -> Self {
    //     let id = UtxoId::new(txid, vout);
    //     let data = UtxoData::new(amount, public_key);
    //     Self { id, data }
    // }

    pub fn id(&self) -> &UtxoId {
        &self.id
    }

    pub fn data(&self) -> &UtxoData {
        &self.data
    }

    pub fn txid(&self) -> &Hash {
        &self.id.txid()
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

    // pub fn serialize(&self) -> Vec<u8> {
    //     self.amount
    //         .to_be_bytes()
    //         .iter()
    //         .chain(self.public_key.to_be_bytes().iter())
    //         .copied()
    //         .collect()
    // }
}

impl Eq for Utxo {}

impl PartialEq for Utxo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

// impl From<(UtxoId, UtxoData)> for Utxo {
//     fn from(u: (UtxoId, UtxoData)) -> Self {
//         Self::new(u.0.txid, u.0.vout, u.1.amount, u.1.public_key)
//     }
// }

// impl From<&[u8]> for Utxo {
//     fn from(bytes: &[u8]) -> Self {
//         let amount = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
//         let puzzle = usize::from_be_bytes(bytes[4..12].try_into().unwrap());
//         Self { amount, puzzle }
//     }
// }

// impl fmt::Display for Utxo {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         // write!(
//         //     f,
//         //     "txid: {:?}\n\
//         //      vout: {}\n\
//         //      amount: {}\n\
//         //      public_key: {}",
//         //     self.txid, self.vout, self.amount, self.public_key
//         // )
//         fmt::Debug::fmt(self, f)
//     }
// }

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
        Self::from(bytes.as_ref())
    }

    pub fn amount(&self) -> u32 {
        self.amount
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }
}

impl From<&[u8]> for UtxoData {
    fn from(bytes: &[u8]) -> Self {
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

pub mod pool;
