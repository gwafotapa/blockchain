use log::info;
use rand::Rng;
use rand_core::RngCore;
use secp256k1::{PublicKey, Secp256k1, SecretKey};

use blockchain::utxo::{UtxoData, UtxoId};
use blockchain::Hash;

pub mod common;

#[test]
fn test_utxo_id_ser_deser() {
    common::log_setup();
    let utxo_id = random_utxo_id(1000);
    let utxo_id2 = UtxoId::deserialize(utxo_id.serialize());
    assert_eq!(utxo_id, utxo_id2);
}

fn random_utxo_id(vout_max: usize) -> UtxoId {
    let txid = random_hash();
    let vout = rand::thread_rng().gen_range(0, vout_max);
    UtxoId::new(txid, vout)
}

fn random_hash() -> Hash {
    let mut hash = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut hash);
    Hash::from(hash)
}

#[test]
fn test_utxo_data_ser_deser() {
    common::log_setup();
    let utxo_data = random_utxo_data(1000000);
    let utxo_data2 = UtxoData::deserialize(utxo_data.serialize());
    assert_eq!(utxo_data, utxo_data2);
}

fn random_utxo_data(amount_max: u32) -> UtxoData {
    let amount = rand::thread_rng().gen_range(0, amount_max);
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
