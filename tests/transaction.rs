use log::{info, warn};
use std::thread;
use std::time::Duration;

use blockchain::common::{Message, NODES};
use blockchain::network::Network;

pub mod common;

#[test]
fn consensus() {
    common::log_setup();

    let nodes = NODES;
    let mut network = Network::random(nodes);
    info!("Network:\n{:?}", network);
    network.run();
    thread::sleep(Duration::from_secs(10));

    info!("Network shutting down");
    network.broadcast(Message::ShutDown);
    let mut nodes = Vec::new();
    for option in network.threads_mut() {
        if let Some(thread) = option.take() {
            nodes.push(thread.join().unwrap());
        }
    }
    if nodes.len() > 0 {
        for i in 0..nodes.len() - 1 {
            assert_eq!(nodes[i].utxo_pool(), nodes[i + 1].utxo_pool());
            assert_eq!(nodes[i].transaction_pool(), nodes[i + 1].transaction_pool());
        }
    }
}
