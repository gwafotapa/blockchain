use std::sync::Once;

use rand::Rng;
use rand_core::RngCore;
use secp256k1::{PublicKey, Secp256k1, SecretKey};

use blockchain::utxo::{UtxoData, UtxoId};
use blockchain::Hash;

static INIT: Once = Once::new();

pub const VOUT_MAX: usize = 10;
pub const AMOUNT_MAX: u32 = 1000000;

pub fn log_setup() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

pub fn random_utxo_id(vout_max: usize) -> UtxoId {
    let txid = random_hash();
    let vout = rand::thread_rng().gen_range(0, vout_max + 1);
    UtxoId::new(txid, vout)
}

fn random_hash() -> Hash {
    let mut hash = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut hash);
    Hash::from(hash)
}

pub fn random_utxo_data(amount_max: u32) -> UtxoData {
    let amount = rand::thread_rng().gen_range(0, amount_max + 1);
    let public_key = random_public_key();
    UtxoData::new(amount, public_key)
}

fn random_public_key() -> PublicKey {
    let mut secret_key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut secret_key);
    let secret_key = SecretKey::from_slice(&secret_key).unwrap();
    let secp = Secp256k1::new();
    PublicKey::from_secret_key(&secp, &secret_key)
}

pub fn random_secret_key() -> SecretKey {
    let mut secret_key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut secret_key);
    SecretKey::from_slice(&secret_key).unwrap()
}
