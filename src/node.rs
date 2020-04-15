use rand::Rng;
use std::convert::TryInto;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};

use crate::chain::Blockchain;
use crate::ledger::Ledger;

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
            true => Some(self.blockchain.mined_block().unwrap().to_bytes()),
        }
    }

    pub fn synchronize(&mut self) {
        match self.rx.try_recv() {
            Ok(data) => match &data[0..1] {
                b"b" => {
                    let height = usize::from_be_bytes(data[1..9].try_into().unwrap());
                    println!("Thread {} received block {}", self.id, height);
                    // TODO2: add block if longer chain
                }
                b"t" => {
                    let sender = unsafe { String::from_utf8_unchecked(data[1..11].to_vec()) };
                    let receiver = unsafe { String::from_utf8_unchecked(data[11..21].to_vec()) };
                    let amount = u32::from_be_bytes(data[21..25].try_into().unwrap());
                    println!(
                        "Thread {} received transaction:\n\
                              sender:   {}\n\
                              receiver: {}\n\
                              amount:   {}\n",
                        self.id, sender, receiver, amount
                    );
                    // TODO1: add transaction
                }
                _ => panic!("Thread {} received invalid data: '{:?}'", self.id, data),
            },
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        }
    }
}
