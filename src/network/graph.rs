use petgraph::algo;
use petgraph::graph::{NodeIndex, UnGraph};
use rand::seq::IteratorRandom;
use rand::seq::SliceRandom;
use rand::Rng;

pub fn random_graph(nodes: usize) -> UnGraph<u32, ()> {
    assert!(nodes > 0, "Empty graph");
    let mut graph = UnGraph::with_capacity(nodes, nodes * (nodes - 1));
    for node in 0..nodes as u32 {
        graph.add_node(node);
    }
    if nodes == 1 {
        return graph;
    }

    let mut rng = rand::thread_rng();
    for i in 0..nodes - 1 {
        let node = NodeIndex::new(i);
        let neighbors = rng.gen_range(1, nodes);
        let current_neighbors = graph.neighbors(node).count();
        if current_neighbors >= neighbors {
            continue;
        }

        for j in (i + 1..nodes).choose_multiple(&mut rng, neighbors - current_neighbors) {
            graph.add_edge(node, NodeIndex::new(j), ());
        }
    }

    let last = nodes - 1;
    if graph.neighbors(NodeIndex::new(last)).next().is_none() {
        let neighbor = rng.gen_range(0, last);
        graph.add_edge(NodeIndex::new(last), NodeIndex::new(neighbor), ());
    }
    graph
}

pub fn random_connected_graph(nodes: usize) -> UnGraph<u32, ()> {
    assert!(nodes > 0, "Empty graph");
    let mut graph = UnGraph::with_capacity(nodes, nodes * (nodes - 1));
    for node in 0..nodes as u32 {
        graph.add_node(node);
    }
    if nodes == 1 {
        return graph;
    }

    let mut rng = rand::thread_rng();
    for i in 0..nodes - 1 {
        let node = NodeIndex::new(i);
        let neighbors = rng.gen_range(1, nodes);
        let current_neighbors = graph.neighbors(node).count();
        if current_neighbors >= neighbors {
            continue;
        }

        for j in (i + 1..nodes).choose_multiple(&mut rng, neighbors - current_neighbors) {
            graph.add_edge(node, NodeIndex::new(j), ());
        }
    }

    let last = nodes - 1;
    if graph.neighbors(NodeIndex::new(last)).next().is_none() {
        let neighbor = rng.gen_range(0, last);
        graph.add_edge(NodeIndex::new(last), NodeIndex::new(neighbor), ());
    }

    let mut cc = algo::tarjan_scc(&graph);
    while cc.len() != 1 {
        let cc1 = cc.pop().unwrap();
        let cc2 = cc[0..(cc.len() - 1)].choose(&mut rng).unwrap();
        let n1 = cc1.choose(&mut rng).unwrap();
        let n2 = cc2.choose(&mut rng).unwrap();
        graph.add_edge(*n1, *n2, ());
    }
    graph
}
