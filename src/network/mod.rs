use rand_core::RngCore;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use std::fmt;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, Barrier, Mutex};
use std::thread::{self, JoinHandle};

use self::graph::Graph;
use crate::node::behaviour::Behaviour;
use crate::node::message::Message;
use crate::node::Node;

pub use self::neighbour::Neighbour;
pub use self::synchronizer::Synchronizer;

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

        let graph = Graph::random_connected(nodes);
        let mut network = Network::with_capacity(nodes);
        let barrier = Arc::new(Barrier::new(nodes));
        let state = Arc::new(Mutex::new(vec![true; nodes]));
        for id in (0..nodes).rev() {
            let public_key = public_keys[id];
            let secret_key = secret_keys[id];
            let sender = senders[id].clone();
            let listener = listeners.pop().unwrap();
            let neighbours = graph[id]
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

    pub fn shut_down(&mut self) {
        while let Some(thread) = self.threads.pop() {
            self.nodes.push(Some(thread.unwrap().join().unwrap()));
        }
    }

    pub fn nodes(&self) -> &Vec<Option<Node>> {
        &self.nodes
    }

    pub fn nodes_as_ref(&self) -> Vec<&Node> {
        self.nodes.iter().filter_map(|o| o.as_ref()).collect()
    }

    pub fn honest_nodes_as_ref(&self) -> Vec<&Node> {
        self.nodes
            .iter()
            .filter_map(|o| o.as_ref().filter(|n| n.integrity() == Behaviour::Honest))
            .collect()
    }

    pub fn threads_mut(&mut self) -> &mut Vec<Option<JoinHandle<Node>>> {
        self.threads.as_mut()
    }
}

pub fn partition<'a, F>(nodes: &Vec<&'a Node>, f: F) -> Vec<Vec<&'a Node>>
where
    F: Fn(&Node, &Node) -> bool,
{
    let mut sets: Vec<Vec<&Node>> = vec![];
    for node in nodes {
        if let Some(set) = sets.iter_mut().find(|set| f(set[0], node)) {
            set.push(node);
        } else {
            sets.push(vec![node]);
        }
    }
    sets
}

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

pub mod graph;
pub mod neighbour;
pub mod synchronizer;
