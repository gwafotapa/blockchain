use rand::seq::IteratorRandom;
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::ops::Index;

type Vertex = usize;
type Neighborhood = HashSet<Vertex>;

#[derive(Debug)]
pub struct Graph(HashMap<Vertex, Neighborhood>);

impl Graph {
    pub fn with_capacity(capacity: usize) -> Self {
        Graph(HashMap::with_capacity(capacity))
    }

    pub fn size(&self) -> usize {
        self.0.len()
    }

    pub fn random_connected(vertices: usize) -> Graph {
        assert!(vertices > 0, "Graph has no vertices");
        let mut graph = Graph::with_capacity(vertices);
        for vertex in 0..vertices {
            graph.insert(vertex, HashSet::new());
        }
        if vertices == 1 {
            return graph;
        }

        let mut rng = rand::thread_rng();
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

    pub fn insert(&mut self, k: Vertex, v: Neighborhood) -> Option<Neighborhood> {
        self.0.insert(k, v)
    }

    pub fn get_mut(&mut self, k: &Vertex) -> Option<&mut Neighborhood> {
        self.0.get_mut(k)
    }

    pub fn as_ref(&self) -> &HashMap<Vertex, Neighborhood> {
        &self.0
    }

    pub fn as_mut(&mut self) -> &mut HashMap<Vertex, Neighborhood> {
        &mut self.0
    }
}

impl Index<Vertex> for Graph {
    type Output = Neighborhood;

    fn index(&self, index: Vertex) -> &Self::Output {
        &self.0[&index]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn random_connected_graph() {
        let vertices = 10;
        let graph = Graph::random_connected(vertices);
        println!("{:?}", graph);
        assert_eq!(graph.size(), vertices);
        for (vertex, neighborhood) in graph.as_ref() {
            assert!(!neighborhood.is_empty());
            assert!(!neighborhood.contains(vertex));
            for &neighbour in neighborhood {
                assert!(graph[neighbour].contains(vertex));
            }
        }
    }
}
