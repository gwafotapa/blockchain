use rand::{seq::SliceRandom, Rng};
use std::collections::{HashMap, HashSet};

type Node = usize;
type Neighborhood = HashSet<Node>;
type Network = HashMap<Node, Neighborhood>;

pub fn generate_network(nodes: usize) -> Network {
    assert!(nodes > 0, "Network has no nodes");
    let mut network = Network::with_capacity(nodes);
    for node in 0..nodes {
        network.insert(node, HashSet::new());
    }
    if nodes == 1 {
        return network;
    }

    let mut rng = rand::thread_rng();
    let mut candidates = Vec::with_capacity(nodes);
    for i in 0..nodes {
        candidates.push(i);
    }
    for node in 0..nodes - 1 {
        let neighbours = rng.gen_range(1, nodes + 1);
        let current_neighbours = network[&node].len();
        if current_neighbours >= neighbours {
            continue;
        }

        for neighbour in
            candidates[node + 1..].choose_multiple(&mut rng, neighbours - current_neighbours)
        {
            network.get_mut(&node).unwrap().insert(*neighbour);
            network.get_mut(neighbour).unwrap().insert(node);
        }
    }

    let last = nodes - 1;
    if network[&last].is_empty() {
        let neighbour = rng.gen_range(0, last);
        network.get_mut(&last).unwrap().insert(neighbour);
        network.get_mut(&neighbour).unwrap().insert(last);
    }
    network
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_network_test() {
        let nodes = 10;
        let network = generate_network(nodes);
        println!("{:?}", network);
        assert_eq!(network.len(), nodes);
        for (node, neighborhood) in &network {
            assert!(!neighborhood.is_empty());
            assert!(!neighborhood.contains(node));
            for neighbour in neighborhood {
                assert!(network[neighbour].contains(node));
            }
        }
    }
}
