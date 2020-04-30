use rand::seq::SliceRandom;
use rand::Rng;
use rand_core::RngCore;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc};
use std::thread::{self, JoinHandle};

use crate::common::Message;
use crate::node::Node;

type Vertex = usize;
type Neighborhood = HashSet<Vertex>;
type Graph = HashMap<Vertex, Neighborhood>;

pub struct Network {
    nodes: Vec<Option<Node>>,
    threads: Vec<Option<JoinHandle<Node>>>,
    senders: Vec<Sender<Arc<Vec<u8>>>>,
}

impl Network {
    pub fn with_capacity(n: usize) -> Network {
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

    pub fn random(nodes: usize) -> Network {
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
            // TODO: give only secret key and let each node derive its public key
        }

        let mut senders = Vec::with_capacity(nodes);
        let mut listeners = Vec::with_capacity(nodes);
        for _node in 0..nodes {
            let (sender, listener) = mpsc::channel();
            senders.push(sender);
            listeners.push(listener);
        }

        let graph = random_graph(nodes);
        let mut network = Network::with_capacity(nodes);
        for id in (0..nodes).rev() {
            let public_key = public_keys[id];
            let secret_key = secret_keys[id];
            let sender = senders[id].clone();
            let listener = listeners.pop().unwrap();
            let neighbours = graph[&id]
                .iter()
                .map(|&x| (x, public_keys[x], senders[x].clone()))
                .collect();
            let node = Node::new(
                id,
                public_key,
                secret_key,
                sender,
                listener,
                neighbours,
                public_keys.clone(),
            );
            network.add(node);
        }
        network
    }

    pub fn run(&mut self) {
        for node in &mut self.nodes {
            let mut node = node.take().unwrap();
            self.threads.push(Some(thread::spawn(move || {
                node.run();
                node
            })));
        }
    }

    pub fn broadcast(&self, message: Message) {
        let bytes = Arc::new(message.serialize());
        for sender in &self.senders {
            sender.send(Arc::clone(&bytes)).unwrap();
        }
    }

    pub fn threads_mut(&mut self) -> &mut Vec<Option<JoinHandle<Node>>> {
        self.threads.as_mut()
    }
}

fn random_graph(vertices: usize) -> Graph {
    assert!(vertices > 0, "Graph has no vertices");
    let mut graph = Graph::with_capacity(vertices);
    for vertex in 0..vertices {
        graph.insert(vertex, HashSet::new());
    }
    if vertices == 1 {
        return graph;
    }

    let mut rng = rand::thread_rng();
    let mut candidates = Vec::with_capacity(vertices);
    for i in 0..vertices {
        candidates.push(i);
    }
    for vertex in 0..vertices - 1 {
        let neighbours = rng.gen_range(1, vertices + 1);
        let current_neighbours = graph[&vertex].len();
        if current_neighbours >= neighbours {
            continue;
        }

        for neighbour in
            candidates[vertex + 1..].choose_multiple(&mut rng, neighbours - current_neighbours)
        {
            graph.get_mut(&vertex).unwrap().insert(*neighbour);
            graph.get_mut(neighbour).unwrap().insert(vertex);
        }
    }

    let last = vertices - 1;
    if graph[&last].is_empty() {
        let neighbour = rng.gen_range(0, last);
        graph.get_mut(&last).unwrap().insert(neighbour);
        graph.get_mut(&neighbour).unwrap().insert(last);
    }
    graph
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
            let neighborhood: Vec<usize> = node.neighbours().iter().map(|x| x.0).collect();
            write!(
                f,
                "Node #{}   Neighbours: {:?}   pk: {}\n",
                node.id(),
                neighborhood,
                node.public_key(),
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_random_graph() {
        let vertices = 10;
        let graph = random_graph(vertices);
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
