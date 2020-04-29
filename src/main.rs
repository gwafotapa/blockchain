/* TODO:
 * 1 - Display txid in hexadecimal form like PublicKey
 * 2 - An utxo is not a couple (txinput, txoutput):
 *     An utxo is a tuple (txid, vout, amount, public_key)
 *     A transcation input is a tuple (txid, vout, signature)
 *     A transaction output is a tuple (amount, public_key)
 * 3 - Implement signatures
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
