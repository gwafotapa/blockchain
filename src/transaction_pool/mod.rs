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

    /// Removes the block transactions (that are in the pool) from the pool
    pub fn process(&mut self, block: &Block) {
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

    pub fn synchronize_with(
        &mut self,
        blockchain: &Blockchain,
        utxo_pool: &UtxoPool,
        fork_block: Option<&Block>,
    ) {
        self.transactions.retain(|tx| {
            !blockchain.contains_tx(tx.id(), fork_block, None)
                && utxo_pool.check_utxos_exist_for(tx).is_ok()
        });
    }

    pub fn undo_all(&mut self, blocks: Vec<Block>) {
        for mut block in blocks {
            for transaction in block.transactions_mut().pop() {
                self.add(transaction).unwrap();
            }
        }
    }

    pub fn recalculate(
        &mut self,
        blocks_to_undo: Vec<Block>,
        blockchain: &Blockchain,
        utxo_pool: &UtxoPool,
    ) {
        let fork_block = blocks_to_undo
            .get(0)
            .map(|b| blockchain.get_parent_of(b))
            .flatten();
        self.undo_all(blocks_to_undo);
        self.synchronize_with(blockchain, utxo_pool, fork_block);
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
