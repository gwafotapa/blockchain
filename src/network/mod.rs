use rand::seq::IteratorRandom;
// use rand::seq::SliceRandom;
use rand::Rng;
use rand_core::RngCore;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, Barrier, Mutex};
use std::thread::{self, JoinHandle};

use crate::node::behaviour::Behaviour;
use crate::node::message::Message;
use crate::node::Node;

pub use self::neighbour::Neighbour;
pub use self::synchronizer::Synchronizer;

type Vertex = usize;
type Neighborhood = HashSet<Vertex>;
type Graph = HashMap<Vertex, Neighborhood>;

pub struct Network {
    nodes: Vec<Option<Node>>,
    threads: Vec<Option<JoinHandle<Node>>>,
    senders: Vec<Sender<Arc<Vec<u8>>>>,
}

impl Network {
    pub fn with_capacity(n: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(n),
            threads: Vec::with_capacity(n),
            senders: Vec::with_capacity(n),
        }
    }

    pub fn add(&mut self, node: Node) {
        self.senders.push(node.sender().clone());
        self.nodes.push(Some(node));
    }

    pub fn random(honest: usize, malicious: usize) -> Self {
        let nodes = honest + malicious;
        let secp = Secp256k1::new();
        let mut rng = rand::thread_rng();
        let mut public_keys = Vec::with_capacity(nodes);
        let mut secret_keys = Vec::with_capacity(nodes);
        for _node in 0..nodes {
            let mut secret_key = [0u8; 32];
            rng.fill_bytes(&mut secret_key);
            let secret_key = SecretKey::from_slice(&secret_key).unwrap();
            let public_key = PublicKey::from_secret_key(&secp, &secret_key);
            secret_keys.push(secret_key);
            public_keys.push(public_key);
        }

        let mut senders = Vec::with_capacity(nodes);
        let mut listeners = Vec::with_capacity(nodes);
        for _node in 0..nodes {
            let (sender, listener) = mpsc::channel();
            senders.push(sender);
            listeners.push(listener);
        }

        let graph = random_connected_graph(nodes);
        let mut network = Network::with_capacity(nodes);
        let barrier = Arc::new(Barrier::new(nodes));
        let state = Arc::new(Mutex::new(vec![true; nodes]));
        for id in (0..nodes).rev() {
            let public_key = public_keys[id];
            let secret_key = secret_keys[id];
            let sender = senders[id].clone();
            let listener = listeners.pop().unwrap();
            let neighbours = graph[&id]
                .iter()
                .map(|&x| Neighbour::new(x, public_keys[x], senders[x].clone()))
                .collect();
            let barrier = Arc::clone(&barrier);
            let state = Arc::clone(&state);
            let synchronizer = Synchronizer::new(barrier, state);
            let integrity = if id < honest {
                Behaviour::Honest
            } else {
                Behaviour::Malicious
            };
            let node = Node::new(
                id,
                public_key,
                secret_key,
                sender,
                listener,
                neighbours,
                public_keys.clone(),
                synchronizer,
                integrity,
            );
            network.add(node);
        }
        network
    }

    pub fn run(&mut self) {
        // for node in &mut self.nodes {
        //     let mut node = node.take().unwrap();
        while let Some(node) = self.nodes.pop() {
            let mut node = node.unwrap();

            let builder = thread::Builder::new().name(node.id().to_string());
            self.threads.push(Some(
                builder
                    .spawn(move || {
                        node.run();
                        node
                    })
                    .unwrap(),
            ));
        }
    }

    pub fn broadcast(&self, message: Message) {
        let bytes = Arc::new(message.serialize());
        for sender in &self.senders {
            sender.send(Arc::clone(&bytes)).unwrap();
        }
    }

    // pub fn consensus(&self) -> Result<(), Vec<Vec<&Node>>> {
    //     let mut cc: Vec<Vec<&Node>> = vec![]; // consensus components
    //     for node in &self.nodes {
    //         let node = node.as_ref().unwrap();
    //         if let Some(c) = cc
    //             .iter_mut()
    //             .find(|c| c[0].blockchain() == node.blockchain())
    //         {
    //             c.push(&node);
    //         } else {
    //             cc.push(vec![&node]);
    //         }
    //     }
    //     if cc.len() == 1 {
    //         Ok(())
    //     } else {
    //         Err(cc)
    //     }
    // }

    // pub fn consensus<F>(&self, f: F) -> Result<(), Vec<Vec<&Node>>>
    // where
    //     F: Fn(&Node, &Node) -> bool,
    // {
    //     let mut cc: Vec<Vec<&Node>> = vec![]; // consensus components
    //     for node in &self.nodes {
    //         let node = node.as_ref().unwrap();
    //         if let Some(c) = cc.iter_mut().find(|c| f(c[0], node)) {
    //             c.push(&node);
    //         } else {
    //             cc.push(vec![&node]);
    //         }
    //     }
    //     if cc.len() == 1 {
    //         Ok(())
    //     } else {
    //         Err(cc)
    //     }
    // }

    // fn blockchain_equality(node1: &Node, node2: &Node) -> bool {
    //     node1.blockchain() == node2.blockchain()
    // }

    // fn utxo_pool_equality(node1: &Node, node2: &Node) -> bool {
    //     node1.utxo_pool() == node2.utxo_pool()
    // }

    // pub fn utxo_pool_consensus(&self) -> Result<(), Vec<Vec<&Node>>> {
    //     let mut cc: Vec<Vec<&Node>> = vec![]; // consensus components
    //     for node in &self.nodes {
    //         let node = node.as_ref().unwrap();
    //         if let Some(c) = cc.iter_mut().find(|c| c[0].utxo_pool() == node.utxo_pool()) {
    //             c.push(&node);
    //         } else {
    //             cc.push(vec![&node]);
    //         }
    //     }
    //     if cc.len() == 1 {
    //         Ok(())
    //     } else {
    //         Err(cc)
    //     }
    // }

    pub fn shut_down(&mut self) {
        while let Some(thread) = self.threads.pop() {
            self.nodes.push(Some(thread.unwrap().join().unwrap()));
        }
    }

    pub fn nodes(&self) -> &Vec<Option<Node>> {
        &self.nodes
    }

    pub fn threads_mut(&mut self) -> &mut Vec<Option<JoinHandle<Node>>> {
        self.threads.as_mut()
    }
}

fn random_connected_graph(vertices: usize) -> Graph {
    assert!(vertices > 0, "Graph has no vertices");
    let mut graph = Graph::with_capacity(vertices);
    for vertex in 0..vertices {
        graph.insert(vertex, HashSet::new());
    }
    if vertices == 1 {
        return graph;
    }

    let mut rng = rand::thread_rng();
    // let candidates: Vec<_> = (0..vertices).collect();
    // for vertex in 0..vertices - 1 {
    //     let neighbours = rng.gen_range(1, vertices + 1);
    //     let current_neighbours = graph[&vertex].len();
    //     if current_neighbours >= neighbours {
    //         continue;
    //     }

    //     for neighbour in
    //         candidates[vertex + 1..].choose_multiple(&mut rng, neighbours - current_neighbours)
    //     {
    //         graph.get_mut(&vertex).unwrap().insert(*neighbour);
    //         graph.get_mut(neighbour).unwrap().insert(vertex);
    //     }
    // }

    // let last = vertices - 1;
    // if graph[&last].is_empty() {
    //     let neighbour = rng.gen_range(0, last);
    //     graph.get_mut(&last).unwrap().insert(neighbour);
    //     graph.get_mut(&neighbour).unwrap().insert(last);
    // }
    for vertex in 1..vertices {
        let neighbours_len = rng.gen_range(1, vertex + 1);
        let neighbours = (0..vertex).choose_multiple(&mut rng, neighbours_len);
        for neighbour in neighbours {
            graph.get_mut(&vertex).unwrap().insert(neighbour);
            graph.get_mut(&neighbour).unwrap().insert(vertex);
        }
    }
    graph
}

pub fn partition<'a, F>(nodes: &Vec<&'a Node>, f: F) -> Vec<Vec<&'a Node>>
where
    F: Fn(&Node, &Node) -> bool,
{
    let mut sets: Vec<Vec<&Node>> = vec![]; // consensus components
    for node in nodes {
        if let Some(set) = sets.iter_mut().find(|set| f(set[0], node)) {
            set.push(node);
        } else {
            sets.push(vec![node]);
        }
    }
    sets
}

// impl Drop for Network {
//     fn drop(&mut self) {
//         info!("Network shutting down");
//         self.broadcast(Message::ShutDown);
//         let mut nodes = Vec::new();
//         for option in &mut self.threads {
//             if let Some(thread) = option.take() {
//                 nodes.push(thread.join().unwrap());
//             }
//         }
//         if nodes.len() > 0 {
//             for i in 0..nodes.len() - 1 {
//                 assert_eq!(nodes[i].utxo_pool(), nodes[i + 1].utxo_pool());
//                 assert_eq!(nodes[i].transaction_pool(), nodes[i + 1].transaction_pool());
//             }
//         }
//     }
// }

impl fmt::Debug for Network {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for node in &self.nodes {
            let node = node.as_ref().unwrap();
            let neighborhood: Vec<usize> = node.neighbours().iter().map(|n| n.id()).collect();
            write!(
                f,
                "Node #{}  integrity: {:?}  pk: {}  Neighbours: {:?}\n",
                node.id(),
                node.integrity(),
                node.public_key(),
                neighborhood,
            )?;
        }

        Ok(())
    }
}

// TODO: remove this module ?
pub mod graph;
pub mod neighbour;
pub mod synchronizer;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_random_connected_graph() {
        let vertices = 10;
        let graph = random_connected_graph(vertices);
        println!("{:?}", graph);
        assert_eq!(graph.len(), vertices);
        for (vertex, neighborhood) in &graph {
            assert!(!neighborhood.is_empty());
            assert!(!neighborhood.contains(vertex));
            for neighbour in neighborhood {
                assert!(graph[neighbour].contains(vertex));
            }
        }
    }
}
