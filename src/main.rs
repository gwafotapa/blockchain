use log::info;
// use rand::Rng;
// use std::ops::Deref;
// use std::sync::mpsc::{self, Receiver, Sender};
// use std::sync::Arc;
use std::thread;
use std::time::Duration;

use blockchain::common::NODES;
use blockchain::network::Network;
// use blockchain::node::Node;

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
    network.run();
    thread::sleep(Duration::from_secs(10));
}
