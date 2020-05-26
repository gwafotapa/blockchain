use crate::error::block::BlockError;
use crate::Hash;

#[derive(Clone, Copy, Debug)]
pub struct Target {
    exponent: u8,
    coefficient: [u8; 3],
}

impl Target {
    pub fn new(exponent: u8, coefficient: [u8; 3]) -> Result<Self, BlockError> {
        if exponent < 3 || 32 < exponent {
            return Err(BlockError::InvalidExponentOfTarget(exponent));
        }
        Ok(Self {
            exponent,
            coefficient,
        })
    }

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
        Self::new(exponent, coefficient).unwrap()
    }
}
