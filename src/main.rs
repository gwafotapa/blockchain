use log::info;
use std::thread;
use std::time::Duration;

use blockchain::constants::NODES;
use blockchain::network::Network;
use blockchain::node::message::Message;

fn main() {
    env_logger::init();

    let mut network = Network::random(NODES, 0);
    info!("Network:\n{:?}", network);

    network.run();
    thread::sleep(Duration::from_secs(5));

    info!("Network shutting down");
    network.broadcast(Message::ShutDown);
    network.shut_down();

    let nodes = network.nodes_as_ref();

    for i in 0..nodes.len() {
        info!("{}", nodes[i]);
    }
}
