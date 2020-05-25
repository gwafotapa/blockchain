// TODO: test transaction processing
// TODO: test transaction undoing
// TODO: test transaction verification: OK
// TODO: test block processing
// TODO: test block validation

use rand::seq::IteratorRandom;
use rand::seq::SliceRandom;
use rand::Rng;
// use secp256k1::{PublicKey, SecretKey};
use std::collections::HashSet;

use blockchain::utxo::Utxo;
// use blockchain::utxo_pool::UtxoPool;
// use blockchain::transaction::{Transaction, TransactionOutput};

pub mod common;

#[test]
fn test_utxo_pool_add_remove() {
    let mut rng = rand::thread_rng();
    let mut utxo_pool = common::random_utxo_pool(None, None);
    let (utxo_id, utxo_data) = utxo_pool.utxos().iter().choose(&mut rng).unwrap();
    let utxo = Utxo::new(*utxo_id, *utxo_data);
    assert!(utxo_pool.add(utxo.clone()).is_err()); // TODO: Check error type
    assert!(utxo_pool.remove(&utxo).is_ok());
    assert!(utxo_pool.remove(&utxo).is_err());
    assert!(utxo_pool.add(utxo).is_ok());
}

#[test]
fn test_utxo_pool_check_double_spending() {
    let (pk, sk) = common::random_key();
    let utxo = common::random_utxo_with(None, None, None, Some(pk));
    let mut utxos = HashSet::new();
    utxos.insert(utxo);
    let utxo_pool = common::random_utxo_pool(Some(utxos), None);
    let tx = common::random_transaction_with(Some(sk), None, Some(vec![utxo]), None);
    assert!(utxo_pool.check_double_spending(&tx).is_ok());

    let tx = common::random_transaction_with(Some(sk), None, Some(vec![utxo, utxo]), None);
    assert!(utxo_pool.check_double_spending(&tx).is_err());
}

#[test]
fn test_utxo_pool_check_utxos_exist() {
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
    let mut utxo_pool = common::random_utxo_pool(Some(utxos), None);
    utxo_pool.remove(&utxo).unwrap();
    let tx = common::random_transaction_with(Some(sk), None, Some(vec![utxo]), None);
    assert!(utxo_pool.check_utxos_exist(&tx).is_err());
}

#[test]
fn test_utxo_pool_check_signatures() {
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
    let utxo_pool = common::random_utxo_pool(Some(utxos), None);
    let tx_utxos_len = rng.gen_range(1, sk1_utxos_len + 1);
    let tx_utxos = sk1_utxos
        .iter()
        .copied()
        .choose_multiple(&mut rng, tx_utxos_len);
    let tx = common::random_transaction_with(Some(sk1), None, Some(tx_utxos), None);
    assert!(utxo_pool.check_signatures(&tx).is_ok());

    let tx_utxos_len = rng.gen_range(1, sk1_utxos_len + 1);
    let mut tx_utxos = sk1_utxos
        .into_iter()
        .choose_multiple(&mut rng, tx_utxos_len);
    let utxo = sk2_utxos.as_slice().choose(&mut rng).copied().unwrap();
    tx_utxos.push(utxo);
    let tx = common::random_transaction_with(Some(sk1), None, Some(tx_utxos), None);
    assert!(utxo_pool.check_signatures(&tx).is_err());
}
