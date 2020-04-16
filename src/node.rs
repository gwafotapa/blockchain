use rand::Rng;
use std::convert::TryInto;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};

use crate::chain::Blockchain;
use crate::ledger::Ledger;
use crate::transaction::Transaction;

const PROBABILITY_NEW_BLOCK: f64 = 1.0 / 1000000.0;

pub struct Node {
    id: usize,
    rx0: Arc<Mutex<Receiver<&'static str>>>,
    txs: Vec<(usize, Sender<Arc<Vec<u8>>>)>,
    rx: Receiver<Arc<Vec<u8>>>,
    ledger: Ledger,
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
            ledger: Ledger::new(),
            blockchain: Blockchain::new(),
        }
    }

    pub fn ledger(&self) -> &Ledger {
        &self.ledger
    }

    pub fn ledger_mut(&mut self) -> &mut Ledger {
        &mut self.ledger
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
            tx.1.send(Arc::clone(&bytes));
        }
    }

    pub fn mine(&mut self) -> Option<Vec<u8>> {
        let mut rng = rand::thread_rng();
        if !self.blockchain.has_mined_block() && !self.ledger.has_next_batch() {
            return None;
        } else if !self.blockchain.has_mined_block() && self.ledger.has_next_batch() {
            self.blockchain
                .set_mined_block(self.ledger.next_batch().unwrap().to_vec());
        }
        match rng.gen_bool(PROBABILITY_NEW_BLOCK) {
            false => None,
            true => {
                let block = self.blockchain.take_mined_block();
                let bytes = block.to_bytes();
                self.ledger.archive(block.transactions());
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
                    // TODO2: add block if longer chain
                }
                b't' => {
                    let transaction = Transaction::from(&data[1..]);
                    println!("Thread {} received {:?}", self.id, transaction);
                    self.ledger.add(vec![transaction]);
                }
                _ => panic!("Thread {} received invalid data: '{:?}'", self.id, data),
            },
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        }
    }
}
