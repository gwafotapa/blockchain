// use log::info;

use rand::seq::IteratorRandom;
use rand::Rng;
use rand_core::RngCore;
use secp256k1::{Message as MessageToSign, PublicKey, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};

use blockchain::transaction::{Transaction, TransactionInput, TransactionOutput};
use blockchain::utxo_pool::UtxoPool;

pub mod common;
pub const NODES: usize = 10;
pub const TRANSACTIONS: usize = 10;

// #[test]
// fn test_utxo_pool_add_remove() {
//     common::log_setup();

//     let keys = generate_keys(NODES);
//     let public_keys = keys.iter().map(|(pk, _sk)| pk).copied().collect();
//     let utxo_pool = UtxoPool::new(public_keys);
//     let transactions = generate_transactions(keys, utxo_pool);
//     let utxo_pool_clone = utxo_pool.clone();
//     utxo_pool.process_all(&transactions);
//     utxo_pool.undo_all(&transactions);
//     assert_eq!(utxo_pool, utxo_pool_clone);
// }

// fn generate_keys(nodes: usize) -> Vec<(PublicKey, SecretKey)> {
//     let mut rng = rand::thread_rng();
//     let secp = Secp256k1::new();
//     let mut keys = Vec::new();
//     for node in 0..nodes {
//         let mut secret_key = [0u8; 32];
//         rng.fill_bytes(&mut secret_key);
//         let secret_key = SecretKey::from_slice(&secret_key).unwrap();
//         let public_key = PublicKey::from_secret_key(&secp, &secret_key);
//         keys.push((public_key, secret_key));
//     }
//     keys
// }

// fn generate_transactions(
//     keys: Vec<(PublicKey, SecretKey)>,
//     utxo_pool: UtxoPool,
// ) -> Vec<Transaction> {
//     let mut rng = rand::thread_rng();
//     let public_keys: Vec<_> = keys.iter().map(|(pk, _sk)| pk).copied().collect();
//     let mut utxos;
//     let mut key;
//     loop {
//         key = keys.iter().choose(&mut rng).unwrap();
//         utxos = utxo_pool.owned_by(&key.0);
//         if !utxos.is_empty() {
//             break;
//         }
//     }

//     let transactions = Vec::new();
//     for i in 0..TRANSACTIONS {
//         let inputs_len = rng.gen_range(1, utxos.len() + 1);
//         let utxos = utxos.iter().choose_multiple(&mut rng, inputs_len);
//         let mut amount: u32 = utxos.iter().map(|u| u.amount()).sum();
//         let mut outputs = Vec::new();
//         loop {
//             let amount1 = rng.gen_range(1, amount + 1);
//             let recipient = *public_keys.iter().choose(&mut rng).unwrap();
//             let output = TransactionOutput::new(amount1, recipient);
//             outputs.push(output);
//             amount -= amount1;
//             if amount == 0 {
//                 break;
//             }
//         }
//         let mut message = Vec::new();
//         for utxo in &utxos {
//             message.extend(utxo.id().serialize());
//         }
//         for output in &outputs {
//             message.extend(output.serialize());
//         }
//         let mut hasher = Sha256::new();
//         hasher.input(message);
//         let hash = hasher.result();
//         let message = MessageToSign::from_slice(&hash).unwrap();
//         let secp = Secp256k1::new();
//         let sig = secp.sign(&message, &key.1);
//         let inputs = utxos
//             .iter()
//             .map(|u| TransactionInput::new(*u.id(), sig))
//             .collect();
//         let transaction = Transaction::new(inputs, outputs);
//         transactions.push(transaction);
//     }
//     transactions
// }
