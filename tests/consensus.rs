use log::info;
use std::collections::HashSet;
use std::thread;
use std::time::Duration;

use blockchain::constants::NODES;
use blockchain::network::{self, Network};
use blockchain::node::message::Message;
use blockchain::utxo::Utxo;

pub mod common;

#[test]
// #[ignore]
fn consensus() {
    common::log_setup();

    let mut network = Network::random(NODES, 0);
    info!("Network:\n{:?}", network);

    network.run();
    thread::sleep(Duration::from_secs(3));

    info!("Network shutting down");
    network.broadcast(Message::ShutDown);
    network.shut_down();

    let nodes = network.nodes_as_ref();

    for i in 0..nodes.len() {
        info!("{}", nodes[i]);
    }

    let mut wallet_utxos_count = 0;
    for i in 0..nodes.len() {
        let wallet: HashSet<Utxo> = nodes[i].wallet().utxos().iter().copied().collect();
        wallet_utxos_count += wallet.len();
        let utxo_pool: HashSet<Utxo> = nodes[i].utxo_pool().into();
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
        assert_eq!(nodes[0].utxo_pool().utxos().len(), wallet_utxos_count);
    } else {
        for set in &sets {
            let subsets = network::partition(set, |n1, n2| n1.utxo_pool() == n2.utxo_pool());
            assert!(subsets.len() == 1);
        }
    }
}

#[test]
// #[ignore]
fn consensus_after_double_spend() {
    common::log_setup();

    let mut network = Network::random(NODES, 1);
    info!("Network:\n{:?}", network);

    network.run();
    thread::sleep(Duration::from_secs(3));

    info!("Network shutting down");
    network.broadcast(Message::ShutDown);
    network.shut_down();

    let honest_nodes = network.honest_nodes_as_ref();

    for i in 0..honest_nodes.len() {
        info!("{}", honest_nodes[i]);
    }

    for i in 0..honest_nodes.len() {
        let wallet: HashSet<Utxo> = honest_nodes[i].wallet().utxos().iter().copied().collect();
        let utxo_pool: HashSet<Utxo> = honest_nodes[i].utxo_pool().into();
        assert!(wallet.is_subset(&utxo_pool));
    }

    let sets = network::partition(&honest_nodes, |n1, n2| n1.blockchain() == n2.blockchain());
    assert!(sets.len() == 1);

    let sets = network::partition(&honest_nodes, |n1, n2| {
        n1.blockchain().top_hash() == n2.blockchain().top_hash()
    });

    if sets.len() == 1 {
        let sets = network::partition(&honest_nodes, |n1, n2| n1.utxo_pool() == n2.utxo_pool());
        assert!(sets.len() == 1);
    } else {
        for set in &sets {
            let subsets = network::partition(set, |n1, n2| n1.utxo_pool() == n2.utxo_pool());
            assert!(subsets.len() == 1);
        }
    }
}
