use rand::seq::IteratorRandom;
use std::collections::HashSet;
use std::fmt;

use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::constants::TXS_PER_BLOCK;
use crate::error::transaction_pool::TransactionPoolError;
use crate::transaction::Transaction;
use crate::utxo_pool::UtxoPool;

#[derive(Debug)]
pub struct TransactionPool {
    transactions: HashSet<Transaction>,
}

impl TransactionPool {
    pub fn new() -> Self {
        Self {
            transactions: HashSet::new(),
        }
    }

    pub fn size(&self) -> usize {
        self.transactions.len()
    }

    pub fn add(&mut self, transaction: Transaction) -> Result<(), TransactionPoolError> {
        if self.transactions.insert(transaction) {
            Ok(())
        } else {
            Err(TransactionPoolError::KnownTransaction)
        }
    }

    pub fn remove(&mut self, transaction: &Transaction) -> Result<(), TransactionPoolError> {
        if self.transactions.remove(transaction) {
            Ok(())
        } else {
            Err(TransactionPoolError::UnknownTransaction)
        }
    }

    // TODO: seems to always be followed by is.ok(). Should it return a bool instead of a result ?
    pub fn compatibility_of(&self, transaction: &Transaction) -> Result<(), TransactionPoolError> {
        for pool_transaction in self.transactions() {
            for pool_input in pool_transaction.inputs() {
                for input in transaction.inputs() {
                    if input.utxo_id() == pool_input.utxo_id() {
                        return Err(TransactionPoolError::UnknownUtxo(*pool_transaction.id()));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn select(&self) -> Option<Vec<Transaction>> {
        if self.size() < TXS_PER_BLOCK {
            return None;
        }
        Some(
            self.transactions
                .iter()
                .choose_multiple(&mut rand::thread_rng(), TXS_PER_BLOCK)
                .iter()
                .map(|&tx| tx.clone())
                .collect(),
        )
    }

    // TODO: redo documentation
    /// Remove the block transactions from the pool
    ///
    /// When a fork occurs, valid received transactions may be deemed invalid if they concern
    /// the other chain. For this reason, there may be no transaction to remove from the pool
    /// when adopting the new chain in the event we lose the race. Hence the ok() call instead
    /// of unwrap().
    pub fn process(&mut self, block: &Block) {
        // for transaction in block.transactions() {
        //     self.remove(transaction).ok();
        // }
        for block_transaction in block.transactions() {
            self.transactions
                .retain(|tx| !tx.shares_utxo_with(block_transaction));
        }
    }

    pub fn process_all(&mut self, blocks: &[Block]) {
        for block in blocks {
            self.process(block);
        }
    }

    // TODO: remove dead code

    // pub fn undo(&mut self, block: &Block) {
    //     for transaction in block.transactions() {
    //         self.add(transaction.clone()).unwrap();
    //     }
    // }

    // pub fn undo_all(&mut self, blocks: &[Block]) {
    //     for block in blocks.iter().rev() {
    //         self.undo(block);
    //     }
    // }

    // TODO? Technically there's no need to go back to the genesis block.
    // Checking the blockchain back to the common parent (of old and new tops) is enough.
    pub fn synchronize_with(&mut self, blockchain: &Blockchain, utxo_pool: &UtxoPool) {
        self.transactions.retain(|tx| {
            !blockchain.contains_tx(tx.id(), None) && utxo_pool.check_utxos_exist(tx).is_ok()
        });
    }

    pub fn undo_all(&mut self, blocks: Vec<Block>) {
        for mut block in blocks {
            for transaction in block.transactions_mut().pop() {
                self.add(transaction).unwrap();
            }
        }
    }

    pub fn transactions(&self) -> &HashSet<Transaction> {
        &self.transactions
    }
}

impl fmt::Display for TransactionPool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Transaction pool ({}) {{\n", self.size())?;
        for transaction in &self.transactions {
            write!(f, "  {:x}\n", transaction.id())?;
        }
        write!(f, "}}\n")
    }
}

impl Eq for TransactionPool {}

impl PartialEq for TransactionPool {
    fn eq(&self, other: &Self) -> bool {
        self.transactions
            .symmetric_difference(&other.transactions)
            .next()
            .is_none()
    }
}
