use std::convert::TryInto;

use crate::common::OUTPUT_SIZE_BYTES;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TransactionOutput {
    amount: u32,
    puzzle: usize,
}

impl TransactionOutput {
    pub fn new(amount: u32, puzzle: usize) -> Self {
        Self { amount, puzzle }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(OUTPUT_SIZE_BYTES);
        bytes.extend(&self.amount.to_be_bytes());
        bytes.extend(&self.puzzle.to_be_bytes());
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

    pub fn puzzle(&self) -> usize {
        self.puzzle
    }
}

impl From<&[u8]> for TransactionOutput {
    fn from(bytes: &[u8]) -> Self {
        let amount = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
        let puzzle = usize::from_be_bytes(bytes[4..12].try_into().unwrap());
        Self { amount, puzzle }
    }
}
