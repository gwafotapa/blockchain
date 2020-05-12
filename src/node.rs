use log::{info, warn};
use secp256k1::{PublicKey, SecretKey};
use std::borrow::Cow;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;

use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::common::Message;
use crate::miner::Miner;
use crate::network::{Neighbour, Synchronizer};
use crate::transaction::{Transaction, TransactionPool};
use crate::utxo::UtxoPool;
use crate::wallet::Wallet;

pub struct Node {
    id: usize,
    public_key: PublicKey,
    secret_key: SecretKey,
    sender: Sender<Arc<Vec<u8>>>,
    listener: Receiver<Arc<Vec<u8>>>,
    neighbours: Vec<Neighbour>,
    blockchain: Blockchain,
    utxo_pool: UtxoPool,
    transaction_pool: TransactionPool,
    wallet: Wallet,
    miner: Miner,
    synchronizer: Synchronizer,
}

impl Node {
    pub fn new(
        id: usize,
        public_key: PublicKey,
        secret_key: SecretKey,
        sender: Sender<Arc<Vec<u8>>>,
        listener: Receiver<Arc<Vec<u8>>>,
        neighbours: Vec<Neighbour>,
        network_public_keys: Vec<PublicKey>,
        synchronizer: Synchronizer,
    ) -> Self {
        let utxo_pool = UtxoPool::new(network_public_keys.clone());
        let wallet = Wallet::new(
            public_key,
            secret_key,
            network_public_keys,
            utxo_pool.owned_by(&public_key),
        );
        Self {
            id,
            public_key,
            secret_key,
            sender,
            listener,
            neighbours,
            blockchain: Blockchain::new(),
            utxo_pool,
            transaction_pool: TransactionPool::new(),
            wallet,
            miner: Miner::new(),
            synchronizer,
        }
    }

    pub fn run(&mut self) {
        loop {
            if let Some(transaction) = self.wallet.initiate() {
                if !self.transaction_pool.contains(&transaction) {
                    info!(
                        "Node #{} --- New transaction:\n{}\n",
                        self.id(),
                        transaction
                    );
                    self.propagate(Message::Transaction(Cow::Borrowed(&transaction)));
                    self.transaction_pool.add(transaction);
                }
            }
            if let Some(block) = self
                .miner
                .mine(self.blockchain.top(), &self.transaction_pool)
            {
                info!("Node #{} --- New block:\n{}\n", self.id, block);
                self.propagate(Message::Block(Cow::Borrowed(&block)));
                self.utxo_pool.process(&block);
                self.wallet.process(&block);
                self.transaction_pool.process(&block);
                self.blockchain.push(block).unwrap();
            }
            if let Ok(bytes) = self.listener.try_recv() {
                match Message::deserialize(bytes.deref()) {
                    Message::Transaction(transaction) => self.process_t(transaction.into_owned()),
                    Message::Block(block) => self.process_b(block.into_owned()),
                    Message::ShutDown => {
                        self.shut_down();
                        return;
                    }
                }
            }
        }
    }

    pub fn process_t(&mut self, transaction: Transaction) {
        if !self.transaction_pool.contains(&transaction)
            && !self.blockchain.contains_t(&transaction)
            && self.utxo_pool.verify(&transaction).is_ok()
        {
            info!(
                "Node #{} --- Received new transaction:\n{}\n",
                self.id, transaction
            );
            self.propagate(Message::Transaction(Cow::Borrowed(&transaction)));
            self.transaction_pool.add(transaction);
        }
    }

    pub fn process_b(&mut self, block: Block) {
        if !self.blockchain.contains(&block) {
            if let Some(parent) = self.blockchain.parent(&block) {
                let (old_blocks, new_blocks) = self.blockchain.block_delta(parent);

                warn!("Node #{} -- old blocks:\n", self.id);
                for block in &old_blocks {
                    warn!("{}", block);
                }
                warn!("Node #{} -- new blocks:\n", self.id);
                for block in &new_blocks {
                    warn!("{}", block);
                }

                self.utxo_pool.undo_all(&old_blocks, &self.blockchain);
                self.utxo_pool.process_all(&new_blocks);
                if self.utxo_pool.validate(&block).is_ok() {
                    info!("Node #{} --- Received new block:\n{}\n", self.id, block);
                    self.propagate(Message::Block(Cow::Borrowed(&block)));
                    if block.height() <= self.blockchain.height() {
                        self.utxo_pool.undo_all(&new_blocks, &self.blockchain);
                        self.utxo_pool.process_all(&old_blocks);
                    } else {
                        self.utxo_pool.process(&block);
                        self.wallet
                            .undo_all(&old_blocks, &self.blockchain, &self.utxo_pool);
                        self.wallet.process_all(&new_blocks);
                        self.wallet.process(&block);
                        self.transaction_pool.undo_all(&old_blocks);
                        self.transaction_pool.process_all(&new_blocks);
                        self.transaction_pool.process(&block);
                        self.miner.discard_block();
                    }
                    self.blockchain.push(block).unwrap();
                } else {
                    self.utxo_pool.undo_all(&new_blocks, &self.blockchain);
                    self.utxo_pool.process_all(&old_blocks);
                }
            }
        }
    }

    pub fn propagate(&self, message: Message) {
        let bytes = Arc::new(message.serialize());
        for neighbour in self.neighbours.iter() {
            neighbour.sender().send(Arc::clone(&bytes)).unwrap();
        }
    }

    pub fn shut_down(&mut self) {
        info!(
            "Node {} shutting down\nPublic key: {}\n",
            // "Node {} shutting down\nPublic key: {}\n\n{}\n{}\n{}\n{}\n",
            self.id,
            self.public_key,
            // self.blockchain,
            // self.transaction_pool,
            // self.utxo_pool,
            // self.wallet
        );
        self.synchronizer.barrier().wait();
        loop {
            let state = self.synchronizer.state();
            let mut state = state.lock().unwrap();
            while let Ok(bytes) = self.listener.try_recv() {
                match Message::deserialize(bytes.deref()) {
                    Message::Transaction(transaction) => self.process_t(transaction.into_owned()),
                    Message::Block(block) => self.process_b(block.into_owned()),
                    Message::ShutDown => panic!("Unexpected shut down message"),
                }
                for neighbour in self.neighbours.iter().map(|n| n.id()) {
                    state[neighbour] = true;
                }
            }
            state[self.id] = false;
            if state.iter().all(|&b| b == false) {
                break;
            }
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    pub fn secret_key(&self) -> &SecretKey {
        &self.secret_key
    }

    pub fn sender(&self) -> &Sender<Arc<Vec<u8>>> {
        &self.sender
    }

    pub fn listener(&self) -> &Receiver<Arc<Vec<u8>>> {
        &self.listener
    }

    pub fn neighbours(&self) -> &Vec<Neighbour> {
        &self.neighbours
    }

    pub fn utxo_pool(&self) -> &UtxoPool {
        &self.utxo_pool
    }

    pub fn transaction_pool(&self) -> &TransactionPool {
        &self.transaction_pool
    }

    pub fn wallet(&self) -> &Wallet {
        &self.wallet
    }

    pub fn blockchain(&self) -> &Blockchain {
        &self.blockchain
    }
}

impl Eq for Node {}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.public_key == other.public_key
    }
}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.public_key.hash(state);
    }
}
