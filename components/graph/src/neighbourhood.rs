use crate::{Backward, DirectedNetworkGraph, Forward, NetworkData, NodeId};
use rayon::prelude::*;

pub type ForwardNeighbourhood = Neighbourhood<Forward>;
pub type BackwardNeighbourhood = Neighbourhood<Backward>;

#[derive(Debug)]
pub struct Neighbourhood<T>
where
    T: NeighbourhoodDirection,
{
    radius: Vec<f32>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Neighbourhood<T>
where
    T: NeighbourhoodDirection,
{
    #[inline]
    pub fn radius(&self, node: NodeId) -> f32 {
        self.radius[node.0 as usize]
    }

    pub fn from_network<D: NetworkData>(size: usize, network: &DirectedNetworkGraph<D>) -> Self {
        T::from_network(size, network)
    }
}

pub trait NeighbourhoodDirection {
    fn find_neighbourhood_radius<D: NetworkData>(
        node: NodeId,
        size: usize,
        network: &DirectedNetworkGraph<D>,
    ) -> Option<f32>;

    fn from_network<D: NetworkData>(
        size: usize,
        network: &DirectedNetworkGraph<D>,
    ) -> Neighbourhood<Self>
    where
        Self: Sized,
    {
        let mut radius = Vec::with_capacity(network.nodes().len());

        network
            .nodes()
            .par_iter()
            .enumerate()
            .map(|(id, _)| Self::find_neighbourhood_radius(id.into(), size, network).unwrap())
            .collect_into_vec(&mut radius);

        Neighbourhood {
            radius,
            _marker: std::marker::PhantomData,
        }
    }
}

impl NeighbourhoodDirection for Forward {
    fn find_neighbourhood_radius<D: NetworkData>(
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
}

impl NeighbourhoodDirection for Backward {
    fn find_neighbourhood_radius<D: NetworkData>(
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
}
