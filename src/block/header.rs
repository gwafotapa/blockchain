use crate::common::Hash;

#[derive(Clone, Debug)]
pub struct BlockHeader {
    hash_prev_block: Hash,
    hash_merkle_root: Hash,
    // nonce: u32,
}

impl BlockHeader {
    pub fn new(hash_prev_block: Hash, hash_merkle_root: Hash) -> Self {
        Self {
            hash_prev_block,
            hash_merkle_root,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.hash_prev_block
            .into_iter()
            .chain(self.hash_merkle_root.into_iter())
            .collect()
    }

    pub fn deserialize<B>(bytes: B) -> Self
    where
        B: AsRef<[u8]>,
    {
        Self::from(bytes)
    }

    pub fn hash_prev_block(&self) -> Hash {
        self.hash_prev_block
    }

    pub fn hash_merkle_root(&self) -> Hash {
        self.hash_merkle_root
    }
}

impl<B> From<B> for BlockHeader
where
    B: AsRef<[u8]>,
{
    fn from(bytes: B) -> Self {
        let bytes = bytes.as_ref();
        let hash_prev_block = *Hash::from_slice(&bytes[..32]);
        let hash_merkle_root = *Hash::from_slice(&bytes[32..]);
        Self {
            hash_prev_block,
            hash_merkle_root,
        }
    }
}
