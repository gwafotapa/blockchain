use generic_array::{typenum::U32, GenericArray};

pub type Hash = GenericArray<u8, U32>;

pub mod block;
pub mod blockchain;
pub mod constants;
pub mod error;
pub mod miner;
pub mod network;
pub mod node;
pub mod transaction;
pub mod transaction_pool;
pub mod utxo;
pub mod utxo_pool;
pub mod wallet;
