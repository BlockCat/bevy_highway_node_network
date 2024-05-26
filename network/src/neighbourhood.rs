use crate::iterators::{DijkstraIterator, Distanceable};
use crate::{HighwayGraph, HighwayNodeIndex};
use petgraph::visit::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Debug)]
pub struct ForwardNeighbourhood(Neighbourhood);

#[derive(Debug)]
pub struct BackwardNeighbourhood(Neighbourhood);

impl Deref for ForwardNeighbourhood {
    type Target = Neighbourhood;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for BackwardNeighbourhood {
    type Target = Neighbourhood;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct Neighbourhood {
    radius: HashMap<HighwayNodeIndex, f32>,
}

impl Neighbourhood {
    #[inline]
    pub fn radius(&self, node: HighwayNodeIndex) -> f32 {
        self.radius[&node]
    }
}

impl ForwardNeighbourhood {
    pub fn from_network<N: Send + Sync, E: Send + Sync + Distanceable>(
        size: usize,
        network: &HighwayGraph<N, E>,
    ) -> Self {
        let mut radius: HashMap<_, _> = network
            .node_identifiers()
            .par_bridge()
            .map(|node| {
                (
                    node,
                    find_forward_neighbourhood_radius(node, size, network).unwrap(),
                )
            })
            .collect();
        radius.shrink_to_fit();
        ForwardNeighbourhood(Neighbourhood { radius })
    }
}

impl BackwardNeighbourhood {
    pub fn from_network<N: Send + Sync, E: Send + Sync + Distanceable>(
        size: usize,
        network: &HighwayGraph<N, E>,
    ) -> Self {
        let mut radius: HashMap<_, _> = network
            .node_identifiers()
            .par_bridge()
            .map(|node| {
                (
                    node,
                    find_backward_neighbourhood_radius(node, size, network).unwrap(),
                )
            })
            .collect();
        radius.shrink_to_fit();
        BackwardNeighbourhood(Neighbourhood { radius })
    }
}

fn find_forward_neighbourhood_radius<N, E: Distanceable>(
    node: HighwayNodeIndex,
    size: usize,
    network: &HighwayGraph<N, E>,
) -> Option<f32> {
    network
        .forward_iterator(node)
        .take(size)
        .last()
        .map(|x| x.1)
}

fn find_backward_neighbourhood_radius<N, E: Distanceable>(
    node: HighwayNodeIndex,
    size: usize,
    network: &HighwayGraph<N, E>,
) -> Option<f32> {
    network
        .backward_iterator(node)
        .take(size)
        .last()
        .map(|x| x.1)
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use crate::{ForwardNeighbourhood, HighwayGraph, HighwayNodeIndex, IntermediateGraph};

    #[test]
    fn forward_neighbourhood_test() {
        // https://www.baeldung.com/wp-content/uploads/2017/01/initial-graph.png
        let mut graph = IntermediateGraph::default();

        let nodes = [
            graph.add_node(0),
            graph.add_node(1),
            graph.add_node(2),
            graph.add_node(3),
            graph.add_node(4),
            graph.add_node(5),
        ];

        graph.add_edge(nodes[0], nodes[1], 10.0); // A => B
        graph.add_edge(nodes[0], nodes[2], 10.0); // A => C
        graph.add_edge(nodes[1], nodes[3], 10.0); // B => D
        graph.add_edge(nodes[1], nodes[5], 10.0); // B => F
        graph.add_edge(nodes[2], nodes[4], 10.0); // C => E
        graph.add_edge(nodes[3], nodes[4], 10.0); // D => E
        graph.add_edge(nodes[3], nodes[5], 10.0); // D => F
        graph.add_edge(nodes[5], nodes[4], 10.0); // F => E

        let graph = HighwayGraph::from(graph);

        let forward = ForwardNeighbourhood::from_network(3, &graph);

        let radius = &forward.radius;

        let map = HashMap::from([
            (HighwayNodeIndex::new(0), 15.0),
            (HighwayNodeIndex::new(1), 6.0),
            (HighwayNodeIndex::new(2), 10.0),
            (HighwayNodeIndex::new(3), 2.0),
            (HighwayNodeIndex::new(4), 0.0),
            (HighwayNodeIndex::new(5), 5.0),
        ]);

        assert_eq!(&map, radius);
    }
}
