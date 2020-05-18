use std::convert::TryInto;

use crate::constants::TARGET;
use crate::Hash;

#[derive(Clone, Debug)]
pub struct BlockHeader {
    hash_prev_block: Hash,
    hash_merkle_root: Hash,
    target: Target,
    nonce: u32,
}

impl BlockHeader {
    pub fn new(hash_prev_block: Hash, hash_merkle_root: Hash) -> Self {
        Self {
            hash_prev_block,
            hash_merkle_root,
            target: TARGET.into(),
            nonce: 0,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.hash_prev_block
            .into_iter()
            .chain(self.hash_merkle_root.into_iter())
            .chain(self.target.serialize())
            .chain(self.nonce.to_be_bytes().to_vec())
            .collect()
    }

    pub fn deserialize<B>(bytes: B) -> Self
    where
        B: AsRef<[u8]>,
    {
        Self::from(bytes)
    }

    pub fn inc_nonce(&mut self) {
        self.nonce += 1;
    }

    pub fn hash_prev_block(&self) -> &Hash {
        &self.hash_prev_block
    }

    pub fn hash_merkle_root(&self) -> &Hash {
        &self.hash_merkle_root
    }

    pub fn target(&self) -> Target {
        self.target
    }
}

impl<B> From<B> for BlockHeader
where
    B: AsRef<[u8]>,
{
    fn from(bytes: B) -> Self {
        let bytes = bytes.as_ref();
        let mut i = 0;
        let hash_prev_block = *Hash::from_slice(&bytes[i..i + 32]);
        i += 32;
        let hash_merkle_root = *Hash::from_slice(&bytes[i..i + 32]);
        i += 32;
        let target = Target::deserialize(&bytes[i..i + 4]);
        i += 4;
        let nonce = u32::from_be_bytes(bytes[i..i + 4].try_into().unwrap());
        Self {
            hash_prev_block,
            hash_merkle_root,
            target,
            nonce,
        }
    }
}

// TODO:
// mod target {
#[derive(Clone, Copy, Debug)]
pub struct Target {
    exponent: u8,
    coefficient: [u8; 3],
}

// TODO: What if exponent (and/or coefficient) is too big or too small (-3)
// trailing zeroes must be less than 256 => exponent must be less than 35
// coefficient bound depends on exponent
impl Target {
    // pub fn new(exponent: u8, coefficient: [u8; 3]) -> Self {
    //     Self {
    //         exponent,
    //         coefficient,
    //     }
    // }

    pub fn hash(&self) -> Hash {
        let mut hash = [0u8; 32];
        let trailing_zeroes = 8 * (self.exponent - 3);
        let mut i = 255 - (trailing_zeroes as usize);
        let mut c = (1 << 16) * self.coefficient[0] as u32
            + (1 << 8) * self.coefficient[1] as u32
            + self.coefficient[2] as u32;
        while c != 0 {
            let bit = (c as u8) & 1;
            hash[i / 8] |= bit << (7 - i % 8);
            c >>= 1;
            i -= 1;
        }
        Hash::from(hash)
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(4);
        vec.push(self.exponent);
        vec.extend(&self.coefficient);
        vec
    }

    pub fn deserialize<B>(bytes: B) -> Self
    where
        B: AsRef<[u8]>,
    {
        Self::from(bytes)
    }
}

impl<B> From<B> for Target
where
    B: AsRef<[u8]>,
{
    fn from(bytes: B) -> Self {
        let bytes = bytes.as_ref();
        let exponent = bytes[0];
        let coefficient = [bytes[1], bytes[2], bytes[3]];
        Self {
            exponent,
            coefficient,
        }
    }
}
// }
