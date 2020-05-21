use std::sync::Once;

use rand::Rng;
use rand_core::RngCore;
use secp256k1::{PublicKey, Secp256k1, SecretKey, Signature};

use blockchain::transaction::{Transaction, TransactionInput, TransactionOutput};
use blockchain::utxo::{UtxoData, UtxoId};
use blockchain::Hash;

static INIT: Once = Once::new();

pub const VOUT_MAX: usize = 10;
pub const AMOUNT_MAX: u32 = 1000000;
pub const INPUTS_LEN_MAX: usize = 10;
pub const OUTPUTS_LEN_MAX: usize = 10;

pub fn log_setup() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

fn random_hash() -> Hash {
    let mut hash = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut hash);
    Hash::from(hash)
}

pub fn random_utxo_id(vout_max: usize) -> UtxoId {
    let txid = random_hash();
    let vout = rand::thread_rng().gen_range(0, vout_max + 1);
    UtxoId::new(txid, vout)
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

pub fn random_transaction_input(vout_max: usize) -> TransactionInput {
    let utxo_id = random_utxo_id(vout_max);
    let mut sig = [0u8; 64];
    rand::thread_rng().fill_bytes(&mut sig);
    let sig = Signature::from_compact(&sig).unwrap();
    TransactionInput::new(utxo_id, sig)
}

fn random_transaction_output(amount_max: u32) -> TransactionOutput {
    TransactionOutput::from(random_utxo_data(amount_max))
}

pub fn random_transaction(
    secret_key: Option<SecretKey>,
    inputs_len: Option<usize>,
    outputs: Option<Vec<TransactionOutput>>,
) -> Transaction {
    let mut rng = rand::thread_rng();
    let secret_key = if let Some(secret_key) = secret_key {
        secret_key
    } else {
        random_secret_key()
    };
    let outputs = if let Some(outputs) = outputs {
        outputs
    } else {
        let outputs_len = rng.gen_range(1, OUTPUTS_LEN_MAX + 1);
        let mut outputs = Vec::with_capacity(outputs_len);
        for _i in 0..outputs_len {
            outputs.push(random_transaction_output(AMOUNT_MAX));
        }
        outputs
    };
    let utxo_ids_len = rng.gen_range(
        1,
        1 + if let Some(length) = inputs_len {
            length
        } else {
            INPUTS_LEN_MAX
        },
    );

    let mut utxo_ids = Vec::with_capacity(utxo_ids_len);
    for _i in 0..utxo_ids_len {
        utxo_ids.push(random_utxo_id(VOUT_MAX));
    }

    Transaction::sign(utxo_ids, outputs, &secret_key)
}
