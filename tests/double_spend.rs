use log::info;
use std::collections::HashSet;
use std::thread;
use std::time::Duration;

use blockchain::constants::NODES;
use blockchain::network::{self, Network};
use blockchain::node::message::Message;
use blockchain::node::Behaviour;
use blockchain::node::Node;
use blockchain::utxo::Utxo;

pub mod common;

#[test]
// #[ignore]
fn consensus_after_double_spend() {
    common::log_setup();

    let mut network = Network::random(NODES, 1);
    info!("Network:\n{:?}", network);

    network.run();
    thread::sleep(Duration::from_secs(1));

    info!("Network shutting down");
    network.broadcast(Message::ShutDown);
    network.shut_down();

    let nodes: Vec<&Node> = network
        .nodes()
        .iter()
        .map(|o| o.as_ref().unwrap())
        .filter(|n| n.integrity() == Behaviour::Honest)
        .collect();

    for i in 0..nodes.len() {
        info!(
            "Node {} shut down\nPublic key: {}\n\n{}\n{}\n{}\n{}\n",
            nodes[i].id(),
            nodes[i].public_key(),
            nodes[i].blockchain(),
            nodes[i].transaction_pool(),
            nodes[i].utxo_pool(),
            nodes[i].wallet()
        );
    }

    for i in 0..nodes.len() {
        let wallet: HashSet<Utxo> = nodes[i].wallet().utxos().iter().copied().collect();
        let utxo_pool: HashSet<Utxo> = nodes[i]
            .utxo_pool()
            .utxos()
            .iter()
            .map(|(id, data)| Utxo::new(*id, *data))
            .collect();
        assert!(wallet.is_subset(&utxo_pool));
    }

    let sets = network::partition(&nodes, |n1, n2| n1.blockchain() == n2.blockchain());
    assert!(sets.len() == 1);

    let sets = network::partition(&nodes, |n1, n2| {
        n1.blockchain().top_hash() == n2.blockchain().top_hash()
    });

    if sets.len() == 1 {
        let sets = network::partition(&nodes, |n1, n2| n1.utxo_pool() == n2.utxo_pool());
        assert!(sets.len() == 1);
    } else {
        for set in &sets {
            let subsets = network::partition(set, |n1, n2| n1.utxo_pool() == n2.utxo_pool());
            assert!(subsets.len() == 1);
        }
    }
}
