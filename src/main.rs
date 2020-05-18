/* TODO:
 * - implement mining proof of work
 * - Modify serialization of objects whose size is known at compile time by returning an array instead of a vec
 * - add enemy nodes
 * - deal with all the TODOs
 * - remove commented code
 * - clean and refactor
 * - pub uses
 * - write/complete tests
 * - if network is not a connected space, tests will very likely fail
 */
use log::info;
use std::thread;
use std::time::Duration;

use blockchain::constants::NODES;
use blockchain::network::Network;
use blockchain::node::message::Message;

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
