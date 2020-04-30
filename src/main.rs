/* TODO:
 * 1 - Factor message signature
 * 2 - Check display and test (add a relay function ?)
 */

use log::info;
// use rand::Rng;
// use std::ops::Deref;
// use std::sync::mpsc::{self, Receiver, Sender};
// use std::sync::Arc;
use std::thread;
use std::time::Duration;

use blockchain::common::{Message, NODES};
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

    info!("Network shutting down");
    network.broadcast(Message::ShutDown);
    for option in network.threads_mut() {
        if let Some(thread) = option.take() {
            thread.join().unwrap();
        }
    }
}
