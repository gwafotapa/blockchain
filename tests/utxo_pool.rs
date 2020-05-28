// TODO: test transaction processing
// TODO: test transaction undoing
// TODO: test transaction verification: OK
// TODO: test block processing
// TODO: test block undoing
// TODO: test block validation
// TODO: make a module directory and split into files (check_transaction.rs, check_block.rs)

use rand::seq::IteratorRandom;
use rand::seq::SliceRandom;
use rand::Rng;
// use secp256k1::{PublicKey, SecretKey};
use std::collections::HashSet;

use blockchain::blockchain::Blockchain;
use blockchain::constants::UTXO_HASH_INIT;
use blockchain::utxo::Utxo;
use blockchain::utxo_pool::UtxoPool;
use blockchain::Hash;
// use blockchain::transaction::{Transaction, TransactionOutput};

pub mod common;

#[test]
fn utxo_pool_add_remove() {
    let mut rng = rand::thread_rng();
    let mut utxo_pool = common::random_utxo_pool(None);
    let (utxo_id, utxo_data) = utxo_pool.utxos().iter().choose(&mut rng).unwrap();
    let utxo = Utxo::new(*utxo_id, *utxo_data);
    assert!(utxo_pool.add(utxo.clone()).is_err()); // TODO: Check error type
    assert!(utxo_pool.remove(&utxo).is_ok());
    assert!(utxo_pool.remove(&utxo).is_err());
    assert!(utxo_pool.add(utxo).is_ok());
}

#[test]
fn utxo_pool_check_utxos_exist() {
    let mut rng = rand::thread_rng();
    let (pk, sk) = common::random_key();
    let sk_utxos_len = rng.gen_range(1, common::UTXOS_PER_KEY_MAX);
    let sk_utxos: Vec<_> = (0..sk_utxos_len)
        .map(|_| common::random_utxo_with(None, None, None, Some(pk)))
        .collect();
    let utxo = sk_utxos.as_slice().choose(&mut rng).copied().unwrap();
    let other_utxos_len = rng.gen_range(0, common::UTXOS_PER_KEY_MAX);
    let other_utxos: Vec<_> = (0..other_utxos_len)
        .map(|_| common::random_utxo(None, None))
        .collect();
    let utxos: HashSet<_> = sk_utxos.into_iter().chain(other_utxos).collect();
    let mut utxo_pool = common::random_utxo_pool(Some(utxos));
    utxo_pool.remove(&utxo).unwrap();
    let tx = common::random_transaction_with(Some(sk), None, Some(vec![utxo]), None);
    assert!(utxo_pool.check_utxos_exist_for(&tx).is_err());
}

#[test]
fn utxo_pool_authenticate() {
    let mut rng = rand::thread_rng();
    let (pk1, sk1) = common::random_key();
    let (mut pk2, mut sk2);
    loop {
        let key = common::random_key();
        pk2 = key.0;
        sk2 = key.1;
        if (pk1, sk1) != (pk2, sk2) {
            break;
        }
    }
    let sk1_utxos_len = rng.gen_range(1, common::UTXOS_PER_KEY_MAX);
    let sk1_utxos: Vec<_> = (0..sk1_utxos_len)
        .map(|_| common::random_utxo_with(None, None, None, Some(pk1)))
        .collect();
    let sk2_utxos_len = rng.gen_range(1, common::UTXOS_PER_KEY_MAX);
    let sk2_utxos: Vec<_> = (0..sk2_utxos_len)
        .map(|_| common::random_utxo_with(None, None, None, Some(pk2)))
        .collect();
    let utxos: HashSet<_> = sk1_utxos.iter().chain(sk2_utxos.iter()).copied().collect();
    let utxo_pool = common::random_utxo_pool(Some(utxos));
    let tx_utxos_len = rng.gen_range(1, sk1_utxos_len + 1);
    let tx_utxos = sk1_utxos
        .iter()
        .copied()
        .choose_multiple(&mut rng, tx_utxos_len);
    let tx = common::random_transaction_with(Some(sk1), None, Some(tx_utxos), None);
    assert!(utxo_pool.authenticate(&tx).is_ok());

    let tx_utxos_len = rng.gen_range(1, sk1_utxos_len + 1);
    let mut tx_utxos = sk1_utxos
        .into_iter()
        .choose_multiple(&mut rng, tx_utxos_len);
    let utxo = sk2_utxos.as_slice().choose(&mut rng).copied().unwrap();
    tx_utxos.push(utxo);
    let tx = common::random_transaction_with(Some(sk1), None, Some(tx_utxos), None);
    assert!(utxo_pool.authenticate(&tx).is_err());
}

#[test]
fn utxo_pool_process_undo_tx() {
    let mut rng = rand::thread_rng();
    let pool_size = rng.gen_range(1, common::UTXO_POOL_SIZE_MAX);
    let utxos: HashSet<_> = (0..pool_size)
        .map(|i| common::random_utxo_with(Some(Hash::from(UTXO_HASH_INIT)), Some(i), None, None))
        .collect();
    let mut utxo_pool = UtxoPool::from(utxos.clone());
    let blockchain = Blockchain::new(utxos.iter().map(|u| (*u.id(), *u.data())).collect());
    let tx_utxos_len = rng.gen_range(1, pool_size + 1);
    let tx_utxos = utxos
        .iter()
        .copied()
        .choose_multiple(&mut rng, tx_utxos_len);
    let tx = common::random_transaction_with(None, None, Some(tx_utxos.clone()), None);
    let utxo_pool_cl = utxo_pool.clone();
    utxo_pool.process_t(&tx);
    assert_eq!(
        utxo_pool.size() + tx.inputs().len(),
        utxo_pool_cl.size() + tx.outputs().len()
    );
    for utxo in tx_utxos {
        assert!(!utxo_pool.contains(&utxo));
    }
    utxo_pool.undo_t(&tx, &blockchain, blockchain.top());
    assert_eq!(utxo_pool, utxo_pool_cl);
}
