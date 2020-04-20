use log::info;
// use rand::Rng;
// use std::convert::TryInto;
use std::hash::{Hash, Hasher};
use std::iter;
use std::ops::Deref;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;

// use crate::block::Block;
// use crate::chain::Blockchain;
// use crate::common::Data;
use crate::common::{Data, NODES};
use crate::transaction::TransactionPool;
use crate::utxo::UtxoPool;
use crate::wallet::Wallet;

// const PROBABILITY_NEW_BLOCK: f64 = 1.0 / 1000000.0;
// const PROBABILITY_NEW_TRANSACTION: f64 = 1.0 / 1000000.0;
// const SEND: usize = 1 << 2;

pub struct Node {
    id: usize,
    neighbours: Vec<(usize, Sender<Arc<Vec<u8>>>)>,
    listener: Receiver<Arc<Vec<u8>>>,
    utxo_pool: UtxoPool,
    transaction_pool: TransactionPool,
    wallet: Wallet,
    // blockchain: Blockchain,
    // rx0: Arc<Mutex<Receiver<&'static str>>>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.id == other.id
    }
}

impl Eq for Node {}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Node {
    pub fn new(
        id: usize,
        neighbours: Vec<(usize, Sender<Arc<Vec<u8>>>)>,
        listener: Receiver<Arc<Vec<u8>>>,
        // rx0: Arc<Mutex<Receiver<&'static str>>>,
    ) -> Self {
        let utxo_pool = UtxoPool::new(NODES);
        let wallet = Wallet::new(utxo_pool.node(id).to_vec());
        Self {
            id,
            neighbours,
            listener,
            utxo_pool,
            transaction_pool: TransactionPool::new(),
            wallet,
            // blockchain: Blockchain::new(),
            // rx0,
        }
    }

    pub fn run(&mut self) {
        loop {
            if let Some(transaction) = self.wallet_mut().manage() {
                self.utxo_pool_mut().process(transaction).unwrap();
                self.transaction_pool_mut().add(transaction);
                info!(
                    "Node #{} --- New transaction:\n{}\n",
                    self.id(),
                    transaction
                );
                self.propagate(Data::Transaction(transaction));
            }
            // if let Some(block) = self.blockchain().mine() {
            //     self.propagate(Data::Block(&block));
            //     self.blockchain.add(block);
            // }
            while let Ok(bytes) = self.listener().try_recv() {
                match Data::from(bytes.deref()) {
                    Data::Transaction(transaction) => {
                        if !self.transaction_pool().contains(transaction) {
                            info!(
                                "Node #{} --- Received transaction:\n{}\n",
                                self.id(),
                                transaction
                            );
                            self.utxo_pool_mut().process(transaction).unwrap();
                            self.transaction_pool_mut().add(transaction);
                            self.propagate(Data::Transaction(transaction));
                        }
                    } //         Data::Block(block) => {
                      //             self.propagate(Data::Block(&block));
                      //             self.blockchain.add(block);
                      //         }
                }
            }
            // match rx0.lock().unwrap().try_recv() {
            //     Ok(SHUT_DOWN) => {
            //         println!("Thread #{} shutting down", id);
            //         break;
            //     }
            //     Ok(message) => panic!("Received unexpected message: \"{}\"", message),
            //     Err(TryRecvError::Empty) => {}
            //     Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
            // }
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn listener(&self) -> &Receiver<Arc<Vec<u8>>> {
        &self.listener
    }

    pub fn utxo_pool(&self) -> &UtxoPool {
        &self.utxo_pool
    }

    pub fn utxo_pool_mut(&mut self) -> &mut UtxoPool {
        &mut self.utxo_pool
    }

    pub fn transaction_pool(&self) -> &TransactionPool {
        &self.transaction_pool
    }

    pub fn transaction_pool_mut(&mut self) -> &mut TransactionPool {
        &mut self.transaction_pool
    }

    pub fn wallet(&self) -> &Wallet {
        &self.wallet
    }

    pub fn wallet_mut(&mut self) -> &mut Wallet {
        &mut self.wallet
    }

    // pub fn blockchain(&self) -> &Blockchain {
    //     &self.blockchain
    // }

    // TODO: try returning &[Transaction]
    // pub fn update_transaction_pool(&mut self) -> Option<&[Transaction]> {
    //     // pub fn update_transaction_pool(&mut self) -> Option<Vec<u8>> {
    //     if let Some(transaction) = Transaction::find(PROBABILITY_NEW_TRANSACTION, &self.utxo_pool) {
    //         self.transaction_pool.add(transaction);
    //     }

    //     if self.transaction_pool.size() < self.transaction_pool.propagated() + SEND {
    //         return None;
    //     }
    //     self.transaction_pool
    //         .set_propagated(self.transaction_pool.size());
    //     Some(self.transaction_pool.transactions())

    //     // let mut bytes = Vec::new();
    //     // bytes.push(b't'); // 't' stands for 'transaction'
    //     // for transaction in &pool.transactions()[pool.propagated()..] {
    //     //     bytes.extend(transaction.serialize());
    //     // }
    //     // pool.set_propagated(pool.size());
    //     // Some(bytes)
    // }

    // pub fn propagate<B>(&self, bytes: B)
    // where
    //     B: Into<Vec<u8>>,
    // {
    //     let bytes = Arc::new(bytes.into());
    //     for tx in self.txs.iter() {
    //         tx.1.send(Arc::clone(&bytes)).unwrap();
    //     }
    // }

    pub fn propagate(&self, data: Data) {
        match data {
            Data::Transaction(transaction) => {
                let bytes = iter::once(b't').chain(transaction.serialize()).collect();
                let bytes = Arc::new(bytes);
                for neighbour in self.neighbours.iter() {
                    neighbour.1.send(Arc::clone(&bytes)).unwrap();
                }
            } // Data::Block(block) => block.serialize(),
        }
    }

    // pub fn mine(&mut self) -> Option<Block> {
    //     let mut rng = rand::thread_rng();
    //     if !self.blockchain.has_mined_block() && !self.transaction_pool.has_next_batch() {
    //         return None;
    //     } else if !self.blockchain.has_mined_block() && self.transaction_pool.has_next_batch() {
    //         self.blockchain
    //             .set_mined_block(self.transaction_pool.next_batch().unwrap().to_vec());
    //     }
    //     match rng.gen_bool(PROBABILITY_NEW_BLOCK) {
    //         false => None,
    //         true => {
    //             let block = self.blockchain.take_mined_block();
    //             Some(block)
    //             // self.transaction_pool.remove(block.transactions());
    //             // self.blockchain.push(block);
    //             // Some(self.blockchain.last())
    //         }
    //     }
    // }

    // pub fn synchronize(&mut self) {
    //     match self.listener.try_recv() {
    //         Ok(data) => match data[0] {
    //             b'b' => {
    //                 let height = usize::from_be_bytes(data[1..9].try_into().unwrap());
    //                 println!("Thread {} received block {}", self.id, height);
    //                 // TODO: add block if longer chain
    //             }
    //             b't' => {
    //                 for bytes in data[1..].chunks(24) {
    //                     let transaction = Transaction::deserialize(bytes);
    //                     println!("Thread {} received {:?}", self.id, transaction);
    //                     self.transaction_pool.add(transaction);
    //                 }
    //             }
    //             _ => panic!("Thread {} received invalid data: '{:?}'", self.id, data),
    //         },
    //         Err(TryRecvError::Empty) => {}
    //         Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
    //     }
    // }
}
