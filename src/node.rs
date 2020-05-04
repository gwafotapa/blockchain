use log::info;
use rand::seq::IteratorRandom;
use rand::Rng;
use secp256k1::{Message as MessageToSign, PublicKey, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;

use crate::chain::Blockchain;
use crate::common::{Message, SPEND_PROBA};
use crate::miner::Miner;
use crate::transaction::{Transaction, TransactionInput, TransactionOutput, TransactionPool};
use crate::utxo::UtxoPool;
use crate::wallet::Wallet;

pub struct Node {
    id: usize,
    public_key: PublicKey,
    secret_key: SecretKey,
    sender: Sender<Arc<Vec<u8>>>,
    listener: Receiver<Arc<Vec<u8>>>,
    neighbours: Vec<(usize, PublicKey, Sender<Arc<Vec<u8>>>)>,
    network: Vec<PublicKey>,
    blockchain: Blockchain,
    utxo_pool: UtxoPool,
    transaction_pool: TransactionPool,
    wallet: Wallet,
    miner: Miner,
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

impl Node {
    pub fn new(
        id: usize,
        public_key: PublicKey,
        secret_key: SecretKey,
        sender: Sender<Arc<Vec<u8>>>,
        listener: Receiver<Arc<Vec<u8>>>,
        neighbours: Vec<(usize, PublicKey, Sender<Arc<Vec<u8>>>)>,
        network: Vec<PublicKey>,
    ) -> Self {
        let blockchain = Blockchain::new();
        let utxo_pool = UtxoPool::new(network.clone());
        let transaction_pool = TransactionPool::new();
        let wallet = Wallet::new(public_key, utxo_pool.owned_by(&public_key));
        let miner = Miner::new(blockchain.top());
        Self {
            id,
            public_key,
            secret_key,
            sender,
            listener,
            neighbours,
            network,
            blockchain,
            utxo_pool,
            transaction_pool,
            wallet,
            miner,
        }
    }

    pub fn run(&mut self) {
        loop {
            if let Some(transaction) = self.initiate() {
                info!(
                    "Node #{} --- New transaction:\n{}\n",
                    self.id(),
                    transaction
                );
                self.propagate(Message::Transaction(Cow::Borrowed(&transaction)));
                self.utxo_pool.process(&transaction);
                self.wallet.process(&transaction);
                self.transaction_pool.add(transaction);
            }
            if let Some(block) = self.miner.mine() {
                info!("Node #{} --- New block:\n{}\n", self.id, block);
                self.propagate(Message::Block(Cow::Borrowed(&block)));
                // self.utxo_pool.process_transactions_from(&block);
                // self.wallet.process_transactions_from(&block);
                // self.transaction_pool.add_transactions_from(&block);
                self.blockchain.push(block);
            }
            if let Ok(message) = self.listener.try_recv() {
                match Message::deserialize(message.deref()) {
                    Message::Transaction(transaction) => {
                        if !self.transaction_pool.contains(&transaction)
                            && self.utxo_pool.verify(&transaction).is_ok()
                        {
                            info!(
                                "Node #{} --- Received new transaction:\n{}\n",
                                self.id, transaction
                            );
                            self.propagate(Message::Transaction(Cow::Borrowed(&transaction)));
                            self.utxo_pool.process(&transaction);
                            self.wallet.process(&transaction);
                            self.transaction_pool.add(transaction.into_owned());
                        }
                    }
                    Message::Block(block) => {
                        if !self.blockchain.contains(&block)
                            && self.utxo_pool.validate(&block).is_ok()
                        {
                            info!("Node #{} --- Received new block:\n{}\n", self.id, block);
                            self.propagate(Message::Block(Cow::Borrowed(&block)));
                            self.blockchain.push(block.into_owned());
                        }
                    }
                    Message::ShutDown => {
                        info!(
                            "Node {} shutting down\n\
                             Transactions: {}\n\
                             Utxo pool:\n\
                             {}",
                            self.id,
                            self.transaction_pool.size(),
                            self.utxo_pool,
                        );
                        return;
                    }
                }
            }
        }
    }

    pub fn initiate(&mut self) -> Option<Transaction> {
        if self.wallet.utxos().is_empty() {
            return None;
        }
        let mut rng = rand::thread_rng();
        match rng.gen_bool(SPEND_PROBA) {
            false => None,
            true => {
                let inputs_len = rng.gen_range(1, self.wallet().utxos().len() + 1);
                let utxos = self
                    .wallet
                    .utxos()
                    .iter()
                    .choose_multiple(&mut rng, inputs_len);
                let mut amount: u32 = utxos.iter().map(|u| u.amount()).sum();
                let mut outputs = Vec::new();
                loop {
                    let amount1 = rng.gen_range(1, amount + 1);
                    let recipient = *self.network.iter().choose(&mut rng).unwrap();
                    let output = TransactionOutput::new(amount1, recipient);
                    outputs.push(output);
                    amount -= amount1;
                    if amount == 0 {
                        break;
                    }
                }
                let mut message = Vec::new();
                for utxo in &utxos {
                    message.extend(utxo.id().serialize());
                }
                for output in &outputs {
                    message.extend(output.serialize());
                }
                let mut hasher = Sha256::new();
                hasher.input(message);
                let hash = hasher.result();
                let message = MessageToSign::from_slice(&hash).unwrap();
                let secp = Secp256k1::new();
                let sig = secp.sign(&message, &self.secret_key);
                let inputs = utxos
                    .iter()
                    .map(|u| TransactionInput::new(*u.id(), sig))
                    .collect();
                let transaction = Transaction::new(inputs, outputs);
                Some(transaction)
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

    pub fn neighbours(&self) -> &[(usize, PublicKey, Sender<Arc<Vec<u8>>>)] {
        &self.neighbours
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
            neighbour.2.send(Arc::clone(&bytes)).unwrap();
        }
    }
}
