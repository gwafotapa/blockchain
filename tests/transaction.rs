use rand::Rng;
use rand_core::RngCore;
use secp256k1::{PublicKey, Secp256k1, SecretKey, Signature};

use blockchain::transaction::{TransactionInput, TransactionOutput};
use blockchain::Hash;

pub mod common;

#[test]
fn test_transaction_input_ser_deser() {
    common::log_setup();
    let transaction_input = random_transaction_input(1000);
    let transaction_input2 = TransactionInput::deserialize(transaction_input.serialize());
    assert_eq!(transaction_input, transaction_input2);
}

fn random_transaction_input(vout_max: usize) -> TransactionInput {
    let utxo_id = common::random_utxo_id(vout_max);
    let mut sig = [0u8; 64];
    rand::thread_rng().fill_bytes(&mut sig);
    let sig = Signature::from_compact(&sig).unwrap();
    TransactionInput::new(utxo_id, sig)
}

#[test]
fn test_transaction_output_ser_deser() {
    let transaction_output = TransactionOutput::from(common::random_utxo_data(1000000));
    let transaction_output2 = TransactionOutput::deserialize(transaction_output.serialize());
    assert_eq!(transaction_output, transaction_output2);
}
