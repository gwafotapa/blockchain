use log::info;
use rand::{seq::SliceRandom, Rng};
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
        let graph = random_graph(nodes);
        let mut senders = Vec::with_capacity(nodes);
        let mut listeners = Vec::with_capacity(nodes);
        for _node in 0..nodes {
            let (sender, listener) = mpsc::channel();
            senders.push(sender);
            listeners.push(listener);
        }
        let mut network = Network::with_capacity(nodes);
        for node in (0..nodes).rev() {
            let sender = senders[node].clone();
            let listener = listeners.pop().unwrap();
            let neighbours = graph[&node]
                .iter()
                .map(|&x| (x, senders[x].clone()))
                .collect();
            let node = Node::new(node, sender, listener, neighbours);
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

impl Drop for Network {
    fn drop(&mut self) {
        info!("Network shutting down");
        self.broadcast(Message::ShutDown);
        let mut nodes = Vec::new();
        for option in &mut self.threads {
            if let Some(thread) = option.take() {
                nodes.push(thread.join().unwrap());
            }
        }
        if nodes.len() > 0 {
            for i in 0..nodes.len() - 1 {
                assert_eq!(nodes[i].utxo_pool(), nodes[i + 1].utxo_pool());
                assert_eq!(nodes[i].transaction_pool(), nodes[i + 1].transaction_pool());
            }
        }
    }
}

impl fmt::Debug for Network {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for node in &self.nodes {
            let node = node.as_ref().unwrap();
            let neighborhood: Vec<usize> = node.neighbours().iter().map(|x| x.0).collect();
            write!(f, "{:?}: {:?}\n", node.id(), neighborhood)?
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
