use log::{info, warn};
use rand::seq::SliceRandom;
use secp256k1::{PublicKey, SecretKey};
use std::borrow::Cow;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;

use self::behaviour::Behaviour;
use self::message::Message;
use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::error::Error;
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
        let utxo_pool = UtxoPool::initialize(network_public_keys.clone());
        let blockchain = Blockchain::new(utxo_pool.utxos().clone());
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
            blockchain,
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
                // && !self.blockchain.contains_tx(transaction.id(), None)
                    && self.blockchain.check_txid_of(&transaction).is_ok()
                {
                    self.process_t(transaction);
                }
            }
            if let Some(block) = self
                .miner
                .mine(self.blockchain.top(), &self.transaction_pool)
            {
                // if !self.blockchain.contains(block.id()) {
                if self.blockchain.check_id_of(&block).is_ok() {
                    // info!("Node #{} --- New block:\n{}\n", self.id, block);
                    // self.propagate(Message::Block(Cow::Borrowed(&block)));
                    // self.utxo_pool.process(&block);
                    // self.wallet.process(&block);
                    // self.transaction_pool.process(&block);
                    // self.blockchain.push(block).unwrap();
                    self.process_b(block, vec![], vec![]);
                }
            }
            if let Ok(bytes) = self.listener.try_recv() {
                match Message::deserialize(bytes.deref()) {
                    Message::Transaction(transaction) => {
                        if self.verify(&transaction).is_ok() {
                            self.process_t(transaction.into_owned())
                        }
                    }
                    Message::Block(block) => {
                        if let Ok((blocks_to_undo, blocks_to_process)) = self.validate(&block) {
                            self.process_b(block.into_owned(), blocks_to_undo, blocks_to_process)
                        }
                    }
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
        // if self.transaction_pool.compatibility_of(&transaction).is_ok()
        //     && !self.blockchain.contains_tx(transaction.id(), None)
        //     && self.utxo_pool.check_utxos_exist_for(&transaction).is_ok()
        //     && self.utxo_pool.authenticate(&transaction).is_ok()
        //     && transaction.check_double_spending().is_ok()
        info!(
            "Node #{} --- Received new transaction:\n{}\n",
            self.id, transaction
        );
        self.propagate(Message::Transaction(Cow::Borrowed(&transaction)));
        self.transaction_pool.add(transaction).unwrap();
    }

    pub fn verify(&self, transaction: &Transaction) -> Result<(), Error> {
        transaction.has_inputs_and_outputs()?;
        transaction.check_double_spending()?;
        self.transaction_pool.compatibility_of(&transaction)?;
        self.blockchain.check_txid_of(transaction)?;
        self.utxo_pool.check_utxos_exist_for(&transaction)?;
        self.utxo_pool.check_balance_of(transaction)?;
        self.utxo_pool.authenticate(&transaction)?;
        Ok(())
    }

    pub fn process_b(
        &mut self,
        block: Block,
        blocks_to_undo: Vec<Block>,
        blocks_to_process: Vec<Block>,
    ) {
        info!("Node #{} --- Received new block:\n{}\n", self.id, block);
        self.propagate(Message::Block(Cow::Borrowed(&block)));
        if block.height() > self.blockchain.height() {
            if block.hash_prev_block() != self.blockchain.top_hash() {
                self.recalculate(blocks_to_undo, blocks_to_process);
            }
            self.utxo_pool.process(&block);
            self.wallet.process(&block);
            self.transaction_pool.process(&block);
            self.miner.discard_block();
        }
        self.blockchain.push(block).unwrap();
    }

    pub fn validate(&mut self, block: &Block) -> Result<(Vec<Block>, Vec<Block>), Error> {
        self.blockchain.check_id_of(block)?;
        self.blockchain.check_txids_of(block)?;
        block.check_transaction_count_is_power_of_two()?;
        block.check_double_spending()?;
        let parent = self.blockchain.parent_of(block)?;
        let (blocks_to_undo, blocks_to_process) =
            self.blockchain.path(self.blockchain.top(), parent);
        self.utxo_pool
            .recalculate(&blocks_to_undo, &blocks_to_process, &self.blockchain);
        self.utxo_pool.check_utxos_exist(block)?;
        self.utxo_pool.check_signatures_of(block)?;
        self.utxo_pool
            .recalculate(&blocks_to_process, &blocks_to_undo, &self.blockchain);
        Ok((blocks_to_undo, blocks_to_process))
    }

    pub fn recalculate(&mut self, blocks_to_undo: Vec<Block>, blocks_to_process: Vec<Block>) {
        self.utxo_pool
            .recalculate(&blocks_to_undo, &blocks_to_process, &self.blockchain);
        self.wallet
            .recalculate(&blocks_to_undo, &blocks_to_process, &self.blockchain);
        self.transaction_pool
            .recalculate(blocks_to_undo, &self.blockchain, &self.utxo_pool);
    }

    // pub fn process_b(&mut self, block: Block) {
    //     if let Some(parent) = self.blockchain.parent(&block) {
    //         let (old_blocks, new_blocks) =
    //             self.blockchain.path(self.blockchain.top(), parent);
    //         self.utxo_pool.undo_all(&old_blocks, &self.blockchain);
    //         self.utxo_pool.process_all(&new_blocks);

    //         if !self.blockchain.contains(block.id())
    //             && self.utxo_pool.validate(&block).is_ok()
    //             && self.blockchain.check_txids_of(&block).is_ok()
    //         {
    //             info!("Node #{} --- Received new block:\n{}\n", self.id, block);
    //             self.propagate(Message::Block(Cow::Borrowed(&block)));
    //             if block.height() <= self.blockchain.height() {
    //                 self.utxo_pool.undo_all(&new_blocks, &self.blockchain);
    //                 self.utxo_pool.process_all(&old_blocks);
    //             } else {
    //                 self.utxo_pool.process(&block);
    //                 self.wallet
    //                     .undo_all(&old_blocks, &self.blockchain, &self.utxo_pool);
    //                 self.wallet.process_all(&new_blocks);
    //                 self.wallet.process(&block);
    //                 self.transaction_pool.undo_all(old_blocks);
    //                 self.transaction_pool
    //                     .synchronize_with(&self.blockchain, &self.utxo_pool);
    //                 self.miner.discard_block();
    //             }
    //             self.blockchain.push(block).unwrap();
    //         } else {
    //             self.utxo_pool.undo_all(&new_blocks, &self.blockchain);
    //             self.utxo_pool.process_all(&old_blocks);
    //         }
    //     }
    // }

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
                    Message::Transaction(transaction) => {
                        if self.verify(&transaction).is_ok() {
                            self.process_t(transaction.into_owned())
                        }
                    }
                    Message::Block(block) => {
                        if let Ok((blocks_to_undo, blocks_to_process)) = self.validate(&block) {
                            self.process_b(block.into_owned(), blocks_to_undo, blocks_to_process)
                        }
                    }
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
                && self.blockchain.check_txid_of(&tx1).is_ok()
                && self.blockchain.check_txid_of(&tx2).is_ok()
            {
                let neighbour1 = neighbours.next().unwrap();
                let neighbour2 = neighbours.next().unwrap();
                warn!(
                    "Node #{} --- Double spend --- New transactions:\n\
                     Sends to: {}\n{}\n\
                     Sends to: {}\n{}\n",
                    self.id(),
                    neighbour1.id(),
                    tx1,
                    neighbour2.id(),
                    tx2
                );
                let msg1 = Message::Transaction(Cow::Borrowed(&tx1));
                let msg2 = Message::Transaction(Cow::Borrowed(&tx2));
                self.send(&msg1, neighbour1);
                self.send(&msg2, neighbour2);
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

    pub fn integrity(&self) -> Behaviour {
        self.integrity
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

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "******* Node {} *******\n\
             \n\
             Public key: {}\n\
             Integrity: {}\n\
             \n\
             {}\n{}\n{}\n{}\n\
             **************\n",
            self.id(),
            self.public_key(),
            self.integrity(),
            self.blockchain(),
            self.transaction_pool(),
            self.utxo_pool(),
            self.wallet()
        )
    }
}

pub mod behaviour;
pub mod message;
