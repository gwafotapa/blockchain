use rand::{seq::SliceRandom, Rng};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::mpsc;

use crate::node::Node;

type Vertex = usize;
type Neighborhood = HashSet<Vertex>;
type Graph = HashMap<Vertex, Neighborhood>;

pub struct Network(pub HashSet<Node>);

impl Network {
    pub fn with_capacity(n: usize) -> Network {
        Self(HashSet::with_capacity(n))
    }

    pub fn insert(&mut self, node: Node) {
        self.0.insert(node);
    }
}

impl fmt::Debug for Network {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for node in &self.0 {
            let neighborhood: Vec<usize> = node.neighbours().iter().map(|x| x.0).collect();
            write!(f, "{:?}: {:?}\n", node.id(), neighborhood)?
        }
        Ok(())
    }
}

pub fn generate_graph(vertices: usize) -> Graph {
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

pub fn generate_network(nodes: usize) -> Network {
    let graph = generate_graph(nodes);
    let mut senders = Vec::with_capacity(nodes);
    let mut receivers = Vec::with_capacity(nodes);
    for _node in 0..nodes {
        let (sender, receiver) = mpsc::channel();
        senders.push(sender);
        receivers.push(receiver);
    }
    let mut network = Network::with_capacity(nodes);
    for node in (0..nodes).rev() {
        let neighbours = graph[&node]
            .iter()
            .map(|x| (*x, senders[*x].clone()))
            .collect();
        let listener = receivers.pop().unwrap();
        let node = Node::new(node, neighbours, listener);
        network.insert(node);
    }
    network
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_graph_test() {
        let vertices = 10;
        let graph = generate_graph(vertices);
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
