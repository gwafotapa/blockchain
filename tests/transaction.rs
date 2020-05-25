use rand::seq::SliceRandom;
use rand::Rng;

use blockchain::transaction::{Transaction, TransactionInput, TransactionOutput};

pub mod common;

#[test]
fn test_transaction_input_ser_deser() {
    let transaction_input = common::random_transaction_input(None, None);
    let transaction_input2 = TransactionInput::deserialize(transaction_input.serialize());
    assert_eq!(transaction_input, transaction_input2);
}

#[test]
fn test_transaction_output_ser_deser() {
    let transaction_output = common::random_transaction_output(None, None);
    let transaction_output2 = TransactionOutput::deserialize(transaction_output.serialize());
    assert_eq!(transaction_output, transaction_output2);
}

#[test]
fn test_transaction_ser_deser() {
    let transaction = common::random_transaction(None, None);
    let (transaction2, transaction2_bytes) = Transaction::deserialize(transaction.serialize());
    assert_eq!(transaction2_bytes, transaction2.bytes());
    assert_eq!(transaction, transaction2);
}

#[test]
fn test_shares_utxo_with() {
    let mut rng = rand::thread_rng();
    let secret_key2 = common::random_secret_key();
    let transaction = common::random_transaction(None, None);
    let shared_utxo_id = *transaction.inputs().choose(&mut rng).unwrap().utxo_id();

    let transaction2 = common::random_transaction_with(Some(secret_key2), None, None, None);
    let mut utxo_ids2: Vec<_> = transaction2.inputs().iter().map(|i| *i.utxo_id()).collect();
    utxo_ids2.insert(
        rng.gen_range(0, transaction2.inputs().len() + 1),
        shared_utxo_id,
    );
    let outputs2 = transaction2.outputs().clone();
    let transaction2 = Transaction::sign(utxo_ids2, outputs2, &secret_key2);
    assert!(transaction.shares_utxo_with(&transaction2));
}
