// use rand::seq::IteratorRandom;
// // use rand::seq::SliceRandom;

// use blockchain::utxo::Utxo;
// // use blockchain::utxo_pool::UtxoPool;
// use blockchain::transaction::{Transaction, TransactionOutput};
// use blockchain::utxo_pool::error::UtxoPoolError;
// use secp256k1::Error::IncorrectSignature;

// pub mod common;

// #[test]
// fn test_utxo_pool_add_remove() {
//     let mut rng = rand::thread_rng();
//     let mut utxo_pool = common::random_utxo_pool(None);
//     let (utxo_id, utxo_data) = utxo_pool.utxos().iter().choose(&mut rng).unwrap();
//     let utxo = Utxo::new(*utxo_id, *utxo_data);
//     assert!(utxo_pool.add(utxo.clone()).is_err());
//     assert!(utxo_pool.remove(&utxo).is_ok());
//     assert!(utxo_pool.remove(&utxo).is_err());
//     assert!(utxo_pool.add(utxo).is_ok());
// }

// #[test]
// fn test_utxo_pool_authenticate() {
//     let mut rng = rand::thread_rng();
//     let keys = common::random_keys(None);
//     let (public_keys, secret_keys): (Vec<_>, Vec<_>) = keys.iter().copied().unzip();
//     let utxo_pool = common::random_utxo_pool(Some(public_keys));
//     let (utxo_id, utxo_data) = utxo_pool.utxos().iter().choose(&mut rng).unwrap();
//     let valid_pk = utxo_data.public_key();
//     let (_, valid_sk) = keys.iter().filter(|(pk, _)| pk == valid_pk).next().unwrap();
//     let transaction = Transaction::sign(
//         vec![*utxo_id],
//         vec![TransactionOutput::from(*utxo_data)],
//         valid_sk,
//     );
//     assert!(utxo_pool.authenticate(&transaction).is_ok());

//     // let mut invalid_sk;
//     // loop {
//     //     invalid_sk = common::random_invalid_sk();
//     //     if !invalid_sks.contains(&invalid_sk) {
//     //         break;
//     //     }
//     // }
//     // let transaction = Transaction::sign(
//     //     vec![*utxo_id],
//     //     vec![TransactionOutput::from(*utxo_data)],
//     //     &invalid_sk,
//     // );
//     // assert_eq!(
//     //     utxo_pool.authenticate(&transaction).unwrap_err(),
//     //     UtxoPoolError::TransactionHasInvalidSignature(IncorrectSignature)
//     // );

//     // let mut unknown_utxo;
//     // loop {
//     //     unknown_utxo_id = common::random_utxo_id(VOUT_MAX);

//     //     let unknown_utxo_data = UtxoData::new(
//     //     if !utxo_pool.contains(&unknown_utxo) {
//     //         break;
//     //     }
//     // }
//     // let transaction = Transaction::sign(
//     //     vec![*unknown_utxo.id()],
//     //     vec![TransactionOutput::from(*unknown_utxo.data())],
//     //     &valid_sk,
//     // );
//     // assert_eq!(
//     //     utxo_pool.authenticate(&transaction).unwrap_err(),
//     //     UtxoPoolError::TransactionHasUnknownUtxo
//     // );
// }
