use log::info;
// use rand::Rng;
use std::ops::Deref;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use blockchain::common::{Data, NODES};
use blockchain::network::Network;
use blockchain::node::Node;

// const MAX_NODES: usize = 10;
const SHUT_DOWN: &str = "Shut down";
// const NODES: usize = 0;
// const PROBA_NEW_TRANSACTION: f64 = 1.0 / 1000.0;

// struct Handler {
//     id: usize,
//     thread: Option<JoinHandle<()>>,
// }

// impl Handler {
//     fn new(
//         id: usize,
//         neighbours: Vec<(usize, Sender<Arc<Vec<u8>>>)>,
//         listener: Receiver<Arc<Vec<u8>>>,
//         // rx0: Arc<Mutex<Receiver<&'static str>>>,
//     ) -> Self {
//         let thread = Some(thread::spawn(move || {
//             // let message = Arc::new(id.to_string());
//             // println!(
//             //     "Starting thread #{} (neighbours: {})",
//             //     id,
//             //     tx.iter().map(|x| &x.0).format(" ")
//             // );

//             // for i in 0..tx.len() {
//             //     tx[i].1.send(Arc::clone(&message)).unwrap();
//             //     println!("Thread #{} sending {} to thread {}", id, message, tx[i].0);
//             // }

//             // TODO: use sender instead of tx; keep tx for transaction
//             // let mut node = Node::new(id, rx0, neighbours, listener);
//             let mut node = Node::new(id, neighbours, listener);
//             loop {
//                 if let Some(transaction) = node.wallet_mut().manage() {
//                     node.utxo_pool_mut().process(transaction).unwrap();
//                     node.transaction_pool_mut().add(transaction);
//                     info!(
//                         "Node #{} --- New transaction:\n{}\n",
//                         node.id(),
//                         transaction
//                     );
//                     node.propagate(Data::Transaction(transaction));
//                 }
//                 // if let Some(block) = node.blockchain().mine() {
//                 //     node.propagate(Data::Block(&block));
//                 //     node.blockchain.add(block);
//                 // }
//                 while let Ok(bytes) = node.listener().try_recv() {
//                     match Data::from(bytes.deref()) {
//                         Data::Transaction(transaction) => {
//                             if !node.transaction_pool().contains(transaction) {
//                                 info!(
//                                     "Node #{} --- Received transaction:\n{}\n",
//                                     node.id(),
//                                     transaction
//                                 );
//                                 node.utxo_pool_mut().process(transaction).unwrap();
//                                 node.transaction_pool_mut().add(transaction);
//                                 node.propagate(Data::Transaction(transaction));
//                             }
//                         } //         Data::Block(block) => {
//                           //             node.propagate(Data::Block(&block));
//                           //             node.blockchain.add(block);
//                           //         }
//                     }
//                 }
//                 // match rx0.lock().unwrap().try_recv() {
//                 //     Ok(SHUT_DOWN) => {
//                 //         println!("Thread #{} shutting down", id);
//                 //         break;
//                 //     }
//                 //     Ok(message) => panic!("Received unexpected message: \"{}\"", message),
//                 //     Err(TryRecvError::Empty) => {}
//                 //     Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
//                 // }
//             }
//         }));
//         Self { id, thread }
//     }
// }

fn main() {
    env_logger::init();
    // let mut rng = rand::thread_rng();
    // let nodes = match NODES {
    //     0 => rng.gen_range(2, MAX_NODES + 1),
    //     value => value,
    // };
    let nodes = NODES;
    let mut network = Network::random(nodes);
    info!("Network:\n{:?}", network);

    // let mut handlers = Vec::with_capacity(nodes);
    // // let (tx0, rx0) = mpsc::channel();
    // // let rx0 = Arc::new(Mutex::new(rx0));

    // let mut senders = Vec::with_capacity(nodes);
    // let mut listeners = Vec::with_capacity(nodes);
    // for _node in 0..nodes {
    //     let (sender, listener) = mpsc::channel();
    //     senders.push(sender);
    //     listeners.push(listener);
    // }
    // for node in (0..nodes).rev() {
    //     let neighbours = network[&node]
    //         .iter()
    //         .map(|x| (*x, senders[*x].clone())) // TODO: Do I need *x ?
    //         .collect();
    //     let listener = listeners.pop().unwrap();
    //     // handlers.push(Handler::new(node, Arc::clone(&rx0), senders, listener));
    //     handlers.push(Handler::new(node, neighbours, listener));
    // }

    let mut threads = Vec::new();
    for mut node in network.take_nodes().unwrap() {
        threads.push(thread::spawn(move || node.run()));
    }

    thread::sleep(Duration::from_secs(3));

    // for _ in &mut handlers {
    //     tx0.send(SHUT_DOWN).unwrap();
    // }
    // for handle in &mut handlers {
    //     if let Some(thread) = handle.thread.take() {
    //         thread.join().unwrap();
    //     }
    // }

    // let terminate = Arc::new(b"shut down".to_vec());
    // for sender in senders {
    //     sender.send(Arc::clone(&terminate)).unwrap();
    // }
    info!("Network shutting down");
    network.broadcast(Data::ShutDown);
    for thread in threads {
        thread.join().unwrap();
    }
}
