use rand::seq::SliceRandom;
use rand::Rng;
use rand_core::RngCore;
use secp256k1::{Message as MessageToSign, Secp256k1, SecretKey, Signature};
use sha2::{Digest, Sha256};

use blockchain::transaction::{Transaction, TransactionInput, TransactionOutput};

pub mod common;

use common::{AMOUNT_MAX, VOUT_MAX};

pub const INPUTS_LEN_MAX: usize = 10;
pub const OUTPUTS_LEN_MAX: usize = 10;

#[test]
fn test_transaction_input_ser_deser() {
    let transaction_input = random_transaction_input(VOUT_MAX);
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
    let transaction_output = TransactionOutput::from(common::random_utxo_data(AMOUNT_MAX));
    let transaction_output2 = TransactionOutput::deserialize(transaction_output.serialize());
    assert_eq!(transaction_output, transaction_output2);
}

#[test]
fn test_transaction_ser_deser() {
    let transaction = random_transaction(None, 10, None);
    let (transaction2, transaction2_bytes) = Transaction::deserialize(transaction.serialize());
    assert_eq!(transaction2_bytes, transaction2.bytes());
    assert_eq!(transaction, transaction2);
}

fn random_transaction(
    sender: Option<SecretKey>,
    inputs_len_max: usize,
    outputs: Option<Vec<TransactionOutput>>,
) -> Transaction {
    let mut rng = rand::thread_rng();
    let sender = if let Some(secret_key) = sender {
        secret_key
    } else {
        common::random_secret_key()
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
    // let amount: u32 = outputs.iter().map(|o| o.amount()).sum();
    let utxo_ids_len = rng.gen_range(1, inputs_len_max + 1);
    let mut utxo_ids = Vec::with_capacity(utxo_ids_len);
    for _i in 0..utxo_ids_len {
        utxo_ids.push(common::random_utxo_id(VOUT_MAX));
    }

    let mut message = Vec::new();
    for utxo_id in &utxo_ids {
        message.extend(utxo_id.serialize());
    }
    for output in &outputs {
        message.extend(output.serialize());
    }
    let mut hasher = Sha256::new();
    hasher.input(message);
    let hash = hasher.result();
    let message = MessageToSign::from_slice(&hash).unwrap();
    let secp = Secp256k1::new();
    let sig = secp.sign(&message, &sender);
    let inputs = utxo_ids
        .iter()
        .map(|id| TransactionInput::new(*id, sig))
        .collect();
    Transaction::new(inputs, outputs)
}

fn random_transaction_output(amount_max: u32) -> TransactionOutput {
    TransactionOutput::from(common::random_utxo_data(amount_max))
}

#[test]
fn test_shares_utxo_with() {
    let mut rng = rand::thread_rng();
    let secret_key2 = common::random_secret_key();
    let transaction = random_transaction(None, 10, None);
    let shared_utxo_id = *transaction.inputs().choose(&mut rng).unwrap().utxo_id();

    let transaction2 = random_transaction(Some(secret_key2), 10, None);
    let mut utxo_ids2: Vec<_> = transaction2
        .inputs()
        .iter()
        .map(|i| i.utxo_id())
        .copied()
        .collect();
    utxo_ids2.insert(
        rng.gen_range(0, transaction2.inputs().len() + 1),
        shared_utxo_id,
    );
    let outputs2 = transaction2.outputs().clone();
    let mut message = Vec::new();
    for utxo_id in &utxo_ids2 {
        message.extend(utxo_id.serialize());
    }
    for output in &outputs2 {
        message.extend(output.serialize());
    }
    let mut hasher = Sha256::new();
    hasher.input(message);
    let hash = hasher.result();
    let message = MessageToSign::from_slice(&hash).unwrap();
    let secp = Secp256k1::new();
    let sig = secp.sign(&message, &secret_key2);
    let inputs2 = utxo_ids2
        .iter()
        .map(|id| TransactionInput::new(*id, sig))
        .collect();
    let transaction2 = Transaction::new(inputs2, outputs2);
    assert!(transaction.shares_utxo_with(&transaction2));
}
