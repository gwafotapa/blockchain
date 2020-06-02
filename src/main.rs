/* TODO:
 * - implement trait Into<HashSet<Utxo>> for type UtxoPool
 * - refactor consensus tests
 * - deal with all the TODOs
 * - remove commented code
 * - clean and refactor
 * - write more tests ?
 * - pub uses ?
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
    let mut network = Network::random(nodes, 0);
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
