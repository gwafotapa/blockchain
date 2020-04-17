use rand::Rng;
use std::convert::TryInto;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};

use crate::chain::Blockchain;
use crate::transaction::{Transaction, TransactionPool};

const PROBABILITY_NEW_BLOCK: f64 = 1.0 / 1000000.0;

pub struct Node {
    id: usize,
    rx0: Arc<Mutex<Receiver<&'static str>>>,
    txs: Vec<(usize, Sender<Arc<Vec<u8>>>)>,
    rx: Receiver<Arc<Vec<u8>>>,
    transaction_pool: TransactionPool,
    blockchain: Blockchain,
}

impl Node {
    pub fn new(
        id: usize,
        rx0: Arc<Mutex<Receiver<&'static str>>>,
        txs: Vec<(usize, Sender<Arc<Vec<u8>>>)>,
        rx: Receiver<Arc<Vec<u8>>>,
    ) -> Self {
        Self {
            id,
            rx0,
            txs,
            rx,
            transaction_pool: TransactionPool::new(),
            blockchain: Blockchain::new(),
        }
    }

    pub fn transaction_pool(&self) -> &TransactionPool {
        &self.transaction_pool
    }

    pub fn transaction_pool_mut(&mut self) -> &mut TransactionPool {
        &mut self.transaction_pool
    }

    pub fn blockchain(&self) -> &Blockchain {
        &self.blockchain
    }

    pub fn propagate<B>(&self, bytes: B)
    where
        B: Into<Vec<u8>>,
    {
        let bytes = Arc::new(bytes.into());
        for tx in self.txs.iter() {
            tx.1.send(Arc::clone(&bytes)).unwrap();
        }
    }

    pub fn mine(&mut self) -> Option<Vec<u8>> {
        let mut rng = rand::thread_rng();
        if !self.blockchain.has_mined_block() && !self.transaction_pool.has_next_batch() {
            return None;
        } else if !self.blockchain.has_mined_block() && self.transaction_pool.has_next_batch() {
            self.blockchain
                .set_mined_block(self.transaction_pool.next_batch().unwrap().to_vec());
        }
        match rng.gen_bool(PROBABILITY_NEW_BLOCK) {
            false => None,
            true => {
                let block = self.blockchain.take_mined_block();
                let bytes = block.to_bytes();
                self.transaction_pool.remove(block.transactions());
                self.blockchain.push(block);
                Some(bytes)
            }
        }
    }

    pub fn synchronize(&mut self) {
        match self.rx.try_recv() {
            Ok(data) => match data[0] {
                b'b' => {
                    let height = usize::from_be_bytes(data[1..9].try_into().unwrap());
                    println!("Thread {} received block {}", self.id, height);
                    // TODO: add block if longer chain
                }
                b't' => {
                    for bytes in data[1..].chunks(24) {
                        let transaction = Transaction::from(bytes);
                        println!("Thread {} received {:?}", self.id, transaction);
                        self.transaction_pool.add(transaction);
                    }
                }
                _ => panic!("Thread {} received invalid data: '{:?}'", self.id, data),
            },
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        }
    }
}
