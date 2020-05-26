use log::info;
use std::collections::HashSet;
use std::thread;
use std::time::Duration;

use blockchain::constants::NODES;
use blockchain::network::Network;
use blockchain::node::message::Message;
use blockchain::utxo::Utxo;

pub mod common;

#[test]
// #[ignore]
fn consensus() {
    common::log_setup();

    let nodes = NODES;
    let mut network = Network::random(nodes, 0);
    info!("Network:\n{:?}", network);
    network.run();
    thread::sleep(Duration::from_secs(5));

    info!("Network shutting down");
    network.broadcast(Message::ShutDown);
    let mut nodes = Vec::new();
    for option in network.threads_mut() {
        if let Some(thread) = option.take() {
            nodes.push(thread.join().unwrap());
        }
    }
    if nodes.len() > 0 {
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
        let mut consensus = true;
        for i in 0..nodes.len() - 1 {
            assert_eq!(nodes[i].blockchain(), nodes[i + 1].blockchain());
            if nodes[i].blockchain().top_hash() != nodes[i + 1].blockchain().top_hash() {
                consensus = false;
                continue;
            }
            assert_eq!(nodes[i].utxo_pool(), nodes[i + 1].utxo_pool());
            assert_eq!(nodes[i].transaction_pool(), nodes[i + 1].transaction_pool());
        }
        if consensus {
            let mut wallet_utxos_count = 0;
            for i in 0..nodes.len() {
                let wallet: HashSet<Utxo> = nodes[i].wallet().utxos().iter().copied().collect();
                wallet_utxos_count += wallet.len();
                let utxo_pool: HashSet<Utxo> = nodes[i]
                    .utxo_pool()
                    .utxos()
                    .iter()
                    .map(|(id, data)| Utxo::new(*id, *data))
                    .collect();
                assert!(wallet.is_subset(&utxo_pool));
            }
            assert_eq!(nodes[0].utxo_pool().utxos().len(), wallet_utxos_count);
        }
    }
}
