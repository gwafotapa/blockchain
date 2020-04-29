use log::info;
use rand::seq::IteratorRandom;
use rand::Rng;
use secp256k1::PublicKey;
use std::borrow::Cow;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;

use crate::common::{Message, PROBABILITY_SPEND};
use crate::transaction::{Transaction, TransactionInput, TransactionOutput, TransactionPool};
use crate::utxo::UtxoPool;
use crate::wallet::Wallet;

pub struct Node {
    public_key: PublicKey,
    sender: Sender<Arc<Vec<u8>>>,
    listener: Receiver<Arc<Vec<u8>>>,
    neighbours: Vec<(PublicKey, Sender<Arc<Vec<u8>>>)>,
    network: Vec<PublicKey>,
    utxo_pool: UtxoPool,
    transaction_pool: TransactionPool,
    wallet: Wallet,
    // blockchain: Blockchain,
    // rx0: Arc<Mutex<Receiver<&'static str>>>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.public_key == other.public_key
    }
}

impl Eq for Node {}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.public_key.hash(state);
    }
}

impl Node {
    pub fn new(
        public_key: PublicKey,
        sender: Sender<Arc<Vec<u8>>>,
        listener: Receiver<Arc<Vec<u8>>>,
        neighbours: Vec<(PublicKey, Sender<Arc<Vec<u8>>>)>,
        network: Vec<PublicKey>, // rx0: Arc<Mutex<Receiver<&'static str>>>,
    ) -> Self {
        let utxo_pool = UtxoPool::new(network.clone());
        let transaction_pool = TransactionPool::new();
        let wallet = Wallet::new(public_key, utxo_pool.owned_by(&public_key));
        Self {
            public_key,
            sender,
            listener,
            neighbours,
            network,
            utxo_pool,
            transaction_pool,
            wallet,
        }
    }

    pub fn run(&mut self) {
        loop {
            if let Some(transaction) = self.initiate() {
                info!(
                    "Node #{} --- New transaction:\n{:?}\n",
                    self.public_key(),
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
                                "Node #{} --- Received transaction:\n{:?}\n",
                                self.public_key(),
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
                            "Node {} shutting down\nTransactions: {}\nUtxo pool: {:?}",
                            self.public_key(),
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
            //         println!("Thread #{} shutting down", public_key);
            //         break;
            //     }
            //     Ok(message) => panic!("Received unexpected message: \"{}\"", message),
            //     Err(TryRecvError::Empty) => {}
            //     Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
            // }
        }
    }

    pub fn initiate(&mut self) -> Option<Transaction> {
        if self.wallet().utxos().is_empty() {
            return None;
        }
        let mut rng = rand::thread_rng();
        match rng.gen_bool(PROBABILITY_SPEND) {
            false => None,
            true => {
                let inputs_len = rng.gen_range(1, self.wallet().utxos().len() + 1);
                let mut inputs = Vec::with_capacity(inputs_len);
                let indices =
                    (0..self.wallet().utxos().len()).choose_multiple(&mut rng, inputs_len);
                // indices.sort_by(|a, b| b.cmp(a));
                let mut amount = 0;
                // for index in indices {
                //     let input = self.utxos.remove(index);
                //     amount += input.amount();
                //     inputs.push(input);
                // }
                for index in indices {
                    let utxo = &self.wallet().utxos()[index];
                    let input = TransactionInput::new(utxo.id().clone());
                    amount += utxo.amount();
                    inputs.push(input);
                }
                let mut outputs = Vec::new();
                loop {
                    let amount1 = rng.gen_range(1, amount + 1);
                    // let mut recipient;
                    // loop {
                    //     recipient = rng.gen_range(0, NODES);
                    //     if recipient != self.public_key {
                    //         break;
                    //     }
                    // }
                    let node = rng.gen_range(0, self.network.len());
                    let recipient = self.network[node];
                    let output = TransactionOutput::new(amount1, recipient);
                    outputs.push(output);
                    amount -= amount1;
                    if amount == 0 {
                        break;
                    }
                }
                let transaction = Transaction::new(inputs, outputs);
                Some(transaction)
            }
        }
    }

    pub fn public_key(&self) -> PublicKey {
        self.public_key
    }

    pub fn sender(&self) -> &Sender<Arc<Vec<u8>>> {
        &self.sender
    }

    pub fn listener(&self) -> &Receiver<Arc<Vec<u8>>> {
        &self.listener
    }

    pub fn neighbours(&self) -> &[(PublicKey, Sender<Arc<Vec<u8>>>)] {
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
    //                 println!("Thread {} received block {}", self.public_key, height);
    //                 // TODO: add block if longer chain
    //             }
    //             b't' => {
    //                 for bytes in message[1..].chunks(24) {
    //                     let transaction = Transaction::deserialize(bytes);
    //                     println!("Thread {} received {:?}", self.public_key, transaction);
    //                     self.transaction_pool.add(transaction);
    //                 }
    //             }
    //             _ => panic!("Thread {} received invalid message: '{:?}'", self.public_key, message),
    //         },
    //         Err(TryRecvError::Empty) => {}
    //         Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
    //     }
    // }
}
