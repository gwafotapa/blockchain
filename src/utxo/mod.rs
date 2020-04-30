use secp256k1::PublicKey;

pub use self::data::UtxoData;
pub use self::id::UtxoId;
use crate::common::Hash;

pub use self::pool::UtxoPool;

#[derive(Clone, Debug)]
pub struct Utxo {
    id: UtxoId,
    data: UtxoData,
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

pub mod data;
pub mod id;
pub mod pool;
