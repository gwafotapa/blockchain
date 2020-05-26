use log::{info, warn};
use rand::seq::SliceRandom;
use secp256k1::{PublicKey, SecretKey};
use std::borrow::Cow;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;

use self::message::Message;
use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::miner::Miner;
use crate::network::{Neighbour, Synchronizer};
use crate::transaction::Transaction;
use crate::transaction_pool::TransactionPool;
use crate::utxo_pool::UtxoPool;
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
    integrity: Behaviour,
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
        integrity: Behaviour,
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
            integrity,
        }
    }

    pub fn run(&mut self) {
        loop {
            if let Some(transaction) = self.wallet.initiate() {
                if self.transaction_pool.compatibility_of(&transaction).is_ok()
                    && !self.blockchain.contains_tx(transaction.id(), None)
                {
                    info!(
                        "Node #{} --- New transaction:\n{}\n",
                        self.id(),
                        transaction
                    );
                    self.propagate(Message::Transaction(Cow::Borrowed(&transaction)));
                    self.transaction_pool.add(transaction).unwrap();
                }
            }
            if let Some(block) = self
                .miner
                .mine(self.blockchain.top(), &self.transaction_pool)
            {
                if !self.blockchain.contains(block.id()) {
                    info!("Node #{} --- New block:\n{}\n", self.id, block);
                    self.propagate(Message::Block(Cow::Borrowed(&block)));
                    self.utxo_pool.process(&block);
                    self.wallet.process(&block);
                    self.transaction_pool.process(&block);
                    self.blockchain.push(block).unwrap();
                }
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
            if self.integrity == Behaviour::Malicious {
                self.double_spend()
            }
        }
    }

    pub fn process_t(&mut self, transaction: Transaction) {
        if self.transaction_pool.compatibility_of(&transaction).is_ok()
            && !self.blockchain.contains_tx(transaction.id(), None)
            && self.utxo_pool.verify(&transaction).is_ok()
        {
            info!(
                "Node #{} --- Received new transaction:\n{}\n",
                self.id, transaction
            );
            self.propagate(Message::Transaction(Cow::Borrowed(&transaction)));
            self.transaction_pool.add(transaction).unwrap();
        }
    }

    // TODO: work on readability
    // is method validate() of UtxoPool properly named ?
    // a check from the blockchain is also needed afterwards!
    // same thing for method verify() of UtxoPool
    pub fn process_b(&mut self, block: Block) {
        if !self.blockchain.contains(block.id()) {
            if let Some(parent) = self.blockchain.parent(&block) {
                let (old_blocks, new_blocks) =
                    self.blockchain.block_delta(self.blockchain.top(), parent);
                // info!("old blocks: {:?}", old_blocks);
                // info!("new blocks: {:?}", new_blocks);
                self.utxo_pool.undo_all(&old_blocks, &self.blockchain);
                self.utxo_pool.process_all(&new_blocks);
                if self.utxo_pool.validate(&block).is_ok()
                    && self.blockchain.check_txids_of(&block).is_ok()
                {
                    // TODO: name validate_transactions
                    // validating a block means:
                    // - checking the id is not already in the blockchain (done)
                    // - checking each utxo is only used once (done)
                    // - checking the number of transactions is power of 2 (done)
                    // - for each transaction:
                    //   - checking its id is not already in the blockchain (done)
                    //   - checking its utxos are unspent (done)
                    //   - checking its signature is valid (done)
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

                        // TODO: Add a function ?
                        self.transaction_pool
                            .synchronize_with(&self.blockchain, &self.utxo_pool);
                        for mut block in old_blocks {
                            while let Some(transaction) = block.transactions_mut().pop() {
                                if !self.blockchain.contains_tx(transaction.id(), None)
                                    && self.utxo_pool.verify(&transaction).is_ok()
                                {
                                    self.transaction_pool.add(transaction).unwrap();
                                }
                            }
                        }
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

    pub fn send(&self, message: &Message, neighbour: &Neighbour) {
        let bytes = Arc::new(message.serialize());
        neighbour.sender().send(Arc::clone(&bytes)).unwrap();
    }

    pub fn shut_down(&mut self) {
        info!(
            "Node {} shutting down\nPublic key: {}\n",
            self.id, self.public_key,
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

    pub fn double_spend(&mut self) {
        let mut rng = rand::thread_rng();
        let mut neighbours = self.neighbours.choose_multiple(&mut rng, 2);
        if neighbours.len() == 1 {
            return;
        }
        if let Some((tx1, tx2)) = self.wallet.double_spend() {
            if self.transaction_pool.compatibility_of(&tx1).is_ok()
                && self.transaction_pool.compatibility_of(&tx2).is_ok()
            {
                warn!(
                    "Node #{} --- Double spend --- New transactions:\n{}\n{}\n",
                    self.id(),
                    tx1,
                    tx2
                );
                let msg1 = Message::Transaction(Cow::Borrowed(&tx1));
                let msg2 = Message::Transaction(Cow::Borrowed(&tx2));
                self.send(&msg1, neighbours.next().unwrap());
                self.send(&msg2, neighbours.next().unwrap());
                self.transaction_pool.add(tx1).unwrap();
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

pub mod message;

#[derive(Eq, PartialEq)]
pub enum Behaviour {
    Honest,
    Malicious,
}
