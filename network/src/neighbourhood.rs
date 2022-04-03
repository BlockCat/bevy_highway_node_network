use std::ops::Deref;

use crate::{DirectedNetworkGraph, NetworkData, NodeId};
use rayon::prelude::*;

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
    radius: Vec<f32>,
}

impl Neighbourhood {
    #[inline]
    pub fn radius(&self, node: NodeId) -> f32 {
        self.radius[node.0 as usize]
    }
}

impl ForwardNeighbourhood {
    pub fn from_network<D: NetworkData>(size: usize, network: &DirectedNetworkGraph<D>) -> Self {
        let mut radius = Vec::with_capacity(network.nodes().len());

        network
            .nodes()
            .par_iter()
            .enumerate()
            .map(|(id, _)| find_forward_neighbourhood_radius(id.into(), size, network).unwrap())
            .collect_into_vec(&mut radius);

        ForwardNeighbourhood(Neighbourhood { radius })
    }
}

impl BackwardNeighbourhood {
    pub fn from_network<D: NetworkData>(size: usize, network: &DirectedNetworkGraph<D>) -> Self {
        let mut radius = Vec::with_capacity(network.nodes().len());

        network
            .nodes()
            .par_iter()
            .enumerate()
            .map(|(id, _)| find_backward_neighbourhood_radius(id.into(), size, network).unwrap())
            .collect_into_vec(&mut radius);

        BackwardNeighbourhood(Neighbourhood { radius })
    }
}

fn find_forward_neighbourhood_radius<D: NetworkData>(
    node: NodeId,
    size: usize,
    network: &DirectedNetworkGraph<D>,
) -> Option<f32> {
    network
        .forward_iterator(node)
        .take(size)
        .last()
        .map(|x| x.1)
}

fn find_backward_neighbourhood_radius<D: NetworkData>(
    node: NodeId,
    size: usize,
    network: &DirectedNetworkGraph<D>,
) -> Option<f32> {
    network
        .backward_iterator(node)
        .take(size)
        .last()
        .map(|x| x.1)
}

#[cfg(test)]
mod tests {

    use crate::{
        create_network,
        tests::{TestEdge, TestNode},
        ForwardNeighbourhood,
    };

    #[test]
    fn forward_neighbourhood_test() {
        // https://www.baeldung.com/wp-content/uploads/2017/01/initial-graph.png
        let network = create_network!(
            0..5,
            0 => 1; 10.0,
            0 => 2; 15.0,
            1 => 3; 5.0,
            1 => 5; 15.0,
            2 => 4; 10.0,
            3 => 4; 2.0,
            3 => 5; 1.0,
            5 => 4; 5.0
        );

        let forward = ForwardNeighbourhood::from_network(3, &network);

        let radius = &forward.radius;

        assert_eq!(&vec![15.0, 6.0, 10.0, 2.0, 0.0, 5.0], radius);
    }
}
