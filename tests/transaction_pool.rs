use blockchain::transaction_pool::TransactionPool;

pub mod common;

#[test]
fn test_transaction_pool_add() {
    let mut transaction_pool = TransactionPool::new();
    let transaction = common::random_transaction(None, None);
    assert!(transaction_pool.add(transaction.clone()).is_ok());
    assert!(transaction_pool.add(transaction.clone()).is_err());
}

#[test]
fn test_transaction_pool_remove() {
    let mut transaction_pool = TransactionPool::new();
    let transaction = common::random_transaction(None, None);
    assert!(transaction_pool.remove(&transaction).is_err());

    transaction_pool.add(transaction.clone()).unwrap();
    assert!(transaction_pool.remove(&transaction).is_ok());
    assert!(transaction_pool.remove(&transaction).is_err());
}

#[test]
fn test_transaction_pool_compatibility_of() {
    let mut transaction_pool = TransactionPool::new();
    let transaction = common::random_transaction(None, None);
    assert!(transaction_pool.compatibility_of(&transaction).is_ok());

    transaction_pool.add(transaction.clone()).unwrap();
    assert!(transaction_pool.compatibility_of(&transaction).is_err());

    transaction_pool.remove(&transaction).unwrap();
    assert!(transaction_pool.compatibility_of(&transaction).is_ok());
}
