use log::info;
use std::borrow::Cow;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;

use crate::common::{Message, NODES};
use crate::transaction::TransactionPool;
use crate::utxo::UtxoPool;
use crate::wallet::Wallet;

pub struct Node {
    id: usize,
    sender: Sender<Arc<Vec<u8>>>,
    listener: Receiver<Arc<Vec<u8>>>,
    neighbours: Vec<(usize, Sender<Arc<Vec<u8>>>)>,
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
        sender: Sender<Arc<Vec<u8>>>,
        listener: Receiver<Arc<Vec<u8>>>,
        neighbours: Vec<(usize, Sender<Arc<Vec<u8>>>)>,
        // rx0: Arc<Mutex<Receiver<&'static str>>>,
    ) -> Self {
        let utxo_pool = UtxoPool::new(NODES);
        let transaction_pool = TransactionPool::new();
        let wallet = Wallet::new(id, utxo_pool.node(id));
        Self {
            id,
            sender,
            listener,
            neighbours,
            utxo_pool,
            transaction_pool,
            wallet,
        }
    }

    pub fn run(&mut self) {
        loop {
            if let Some(transaction) = self.wallet_mut().initiate() {
                info!(
                    "Node #{} --- New transaction:\n{}\n",
                    self.id(),
                    transaction
                );
                self.utxo_pool_mut().process(&transaction).unwrap();
                self.wallet_mut().process(&transaction);
                self.propagate(Message::Transaction(Cow::Borrowed(&transaction)));
                self.transaction_pool_mut().add(transaction);
            }
            if let Ok(message) = self.listener().try_recv() {
                // if let Ok(messages) = self.listener().try_recv() {
                //     for message in Message::from(messages.deref()) {
                //         match message {
                match Message::deserialize(message.deref()) {
                    Message::Transaction(transaction) => {
                        if !self.transaction_pool().contains(&transaction) {
                            info!(
                                "Node #{} --- Received transaction:\n{}\n",
                                self.id(),
                                transaction
                            );
                            self.utxo_pool_mut().process(&transaction).unwrap();
                            self.wallet_mut().process(&transaction);
                            self.propagate(Message::Transaction(Cow::Borrowed(&transaction)));
                            self.transaction_pool_mut().add(transaction.into_owned());
                            // } else {
                            //     info!("Transaction already in the pool");
                        }
                    }
                    Message::ShutDown => {
                        info!(
                            "Node {} shutting down\nTransactions: {}\nUtxo pool: {}",
                            self.id(),
                            self.transaction_pool().size(),
                            self.utxo_pool(),
                        );
                        return;
                    }
                }
                // }
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

    pub fn sender(&self) -> &Sender<Arc<Vec<u8>>> {
        &self.sender
    }

    pub fn listener(&self) -> &Receiver<Arc<Vec<u8>>> {
        &self.listener
    }

    pub fn neighbours(&self) -> &[(usize, Sender<Arc<Vec<u8>>>)] {
        self.neighbours.as_ref()
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

    pub fn propagate(&self, message: Message) {
        let bytes = Arc::new(message.serialize());
        for neighbour in self.neighbours.iter() {
            neighbour.1.send(Arc::clone(&bytes)).unwrap();
        }
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
    //         Ok(message) => match message[0] {
    //             b'b' => {
    //                 let height = usize::from_be_bytes(message[1..9].try_into().unwrap());
    //                 println!("Thread {} received block {}", self.id, height);
    //                 // TODO: add block if longer chain
    //             }
    //             b't' => {
    //                 for bytes in message[1..].chunks(24) {
    //                     let transaction = Transaction::deserialize(bytes);
    //                     println!("Thread {} received {:?}", self.id, transaction);
    //                     self.transaction_pool.add(transaction);
    //                 }
    //             }
    //             _ => panic!("Thread {} received invalid message: '{:?}'", self.id, message),
    //         },
    //         Err(TryRecvError::Empty) => {}
    //         Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
    //     }
    // }
}
