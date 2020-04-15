// use log::info;
use itertools::Itertools;
use rand::Rng;
use std::{
    sync::{
        mpsc::{self, Receiver, Sender, TryRecvError},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use blockchain::{ledger::Ledger, network, node::Node, transaction::Transaction};

const MAX_NODES: usize = 10;
const SHUT_DOWN: &str = "Shut down";
const NODES: usize = 0;
const PROBA_NEW_TRANSACTION: f64 = 1.0 / 1000.0;

struct Handler {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Handler {
    fn new(
        id: usize,
        rx0: Arc<Mutex<Receiver<&'static str>>>,
        // tx: Vec<(usize, Sender<Arc<String>>)>,
        // rx: Receiver<Arc<String>>,
        txs: Vec<(usize, Sender<Arc<Vec<u8>>>)>,
        rx: Receiver<Arc<Vec<u8>>>,
    ) -> Self {
        let thread = Some(thread::spawn(move || {
            // let message = Arc::new(id.to_string());
            // println!(
            //     "Starting thread #{} (neighbours: {})",
            //     id,
            //     tx.iter().map(|x| &x.0).format(" ")
            // );

            // for i in 0..tx.len() {
            //     tx[i].1.send(Arc::clone(&message)).unwrap();
            //     println!("Thread #{} sending {} to thread {}", id, message, tx[i].0);
            // }

            // let mut ledger = Ledger::new();
            let mut node = Node::new(id, rx0, txs, rx);
            loop {
                // ledger.update(PROBA_NEW_TRANSACTION);
                // ledger.send(&tx);
                if let Some(transactions) = node.ledger_mut().update() {
                    // if let Some(transactions) = node.update_ledger() {
                    node.propagate(transactions);
                }
                if let Some(block) = node.mine() {
                    node.propagate(block);
                }
                node.synchronize();

                // match rx.try_recv() {
                //     // Ok(message) => println!("Thread #{} receiving {}", id, message),
                //     Ok(transactions) => {
                //         println!("Thread #{} receiving transactions {:?}", id, transactions)
                //     }
                //     Err(TryRecvError::Empty) => {}
                //     Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
                // }
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
        }));
        Self { id, thread }
    }
}

fn main() {
    let mut rng = rand::thread_rng();
    let nodes = match NODES {
        0 => rng.gen_range(2, MAX_NODES + 1),
        value => value,
    };
    let network = network::generate_network(nodes);
    println!("{:?}", network);

    let mut handlers = Vec::with_capacity(nodes);
    let (tx0, rx0) = mpsc::channel();
    let rx0 = Arc::new(Mutex::new(rx0));

    let mut senders = Vec::with_capacity(nodes);
    let mut receivers = Vec::with_capacity(nodes);
    for _node in 0..nodes {
        let (tx, rx) = mpsc::channel();
        senders.push(tx);
        receivers.push(rx);
    }
    for node in (0..nodes).rev() {
        let txs = network[&node]
            .iter()
            .map(|x| (*x, senders[*x].clone()))
            .collect();
        let rx = receivers.pop().unwrap();
        handlers.push(Handler::new(node, Arc::clone(&rx0), txs, rx));
    }

    thread::sleep(Duration::from_secs(5));

    for _ in &mut handlers {
        tx0.send(SHUT_DOWN).unwrap();
    }
    for handle in &mut handlers {
        if let Some(thread) = handle.thread.take() {
            thread.join().unwrap();
        }
    }
}
