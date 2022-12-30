use petgraph::{
    data::Build,
    graph::Frozen,
    stable_graph::{
        EdgeIndex, EdgeReference, EdgeReferences, Edges, IndexType, NodeIndex, NodeIndices,
        NodeReferences, StableDiGraph,
    },
    visit::{self, IntoEdgeReferences, IntoNodeReferences},
    Direction, IntoWeightedEdge,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ops::Index, slice::SliceIndex};

use crate::iterators::Distanceable;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountStableGraph<N, E, Ix: IndexType> {
    pub graph: StableDiGraph<N, E, Ix>,
    out_edge: HashMap<NodeIndex<Ix>, usize>,
    in_edge: HashMap<NodeIndex<Ix>, usize>,
}

impl<N, E, Ix: IndexType> Default for CountStableGraph<N, E, Ix> {
    fn default() -> Self {
        Self {
            graph: Default::default(),
            out_edge: Default::default(),
            in_edge: Default::default(),
        }
    }
}

impl<N, E: Distanceable, Ix: IndexType> CountStableGraph<N, E, Ix> {
    pub fn update_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> EdgeIndex<Ix> {
        if let Some(ix) = self.find_edge(a, b) {
            let ow = self.graph[ix].distance();
            if weight.distance() < ow {
                self.graph[ix] = weight;
            }
            return ix;
        }
        self.add_edge(a, b, weight)
    }
}

impl<N, E, Ix: IndexType> CountStableGraph<N, E, Ix> {
    pub fn edges_directed(
        &self,
        n: NodeIndex<Ix>,
        dir: Direction,
    ) -> Edges<E, petgraph::Directed, Ix> {
        self.graph.edges_directed(n, dir)
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    pub fn edge_count_out(&self, n: NodeIndex<Ix>) -> usize {
        self.out_edge[&n]
    }
    pub fn edge_count_in(&self, n: NodeIndex<Ix>) -> usize {
        self.in_edge[&n]
    }

    pub fn add_node(&mut self, weight: N) -> NodeIndex<Ix> {
        let n = self.graph.add_node(weight);

        self.out_edge.insert(n, 0);
        self.in_edge.insert(n, 0);

        n
    }

    pub fn recount(&mut self) {
        self.out_edge = self
            .graph
            .node_indices()
            .map(|n| (n, self.graph.edges_directed(n, Direction::Outgoing).count()))
            .collect();
        self.in_edge = self
            .graph
            .node_indices()
            .map(|n| (n, self.graph.edges_directed(n, Direction::Incoming).count()))
            .collect();
    }

    pub fn add_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> EdgeIndex<Ix> {
        let i = self.graph.add_edge(a, b, weight);
        *self.out_edge.entry(a).or_default() += 1;
        *self.in_edge.entry(b).or_default() += 1;
        i
    }

    pub fn remove_node(&mut self, n: NodeIndex<Ix>) -> Option<N> {
        self.graph
            .neighbors_directed(n, Direction::Outgoing)
            .for_each(|n| {
                *self.in_edge.get_mut(&n).unwrap() -= 1;
            });

        self.graph
            .neighbors_directed(n, Direction::Incoming)
            .for_each(|n| {
                *self.out_edge.get_mut(&n).unwrap() -= 1;
            });
        let o = self.graph.remove_node(n);

        self.out_edge.remove(&n);
        self.in_edge.remove(&n);

        o
    }

    pub fn extend_with_edges<I>(&mut self, iterable: I)
    where
        I: IntoIterator,
        I::Item: IntoWeightedEdge<E>,
        <I::Item as IntoWeightedEdge<E>>::NodeId: Into<NodeIndex<Ix>>,
        N: Default,
    {
        let iter = iterable.into_iter();

        for elt in iter {
            let (source, target, weight) = elt.into_weighted_edge();
            let (source, target) = (source.into(), target.into());

            self.add_edge(source, target, weight);
        }
    }

    pub fn map<'a, F, G, N2, E2>(&'a self, node_map: F, edge_map: G) -> CountStableGraph<N2, E2, Ix>
    where
        F: FnMut(NodeIndex<Ix>, &'a N) -> N2,
        G: FnMut(EdgeIndex<Ix>, &'a E) -> E2,
    {
        let graph = self.graph.map(node_map, edge_map);

        CountStableGraph {
            graph,
            in_edge: self.in_edge.clone(),
            out_edge: self.out_edge.clone(),
        }
    }

    pub fn retain_edges<F>(&mut self, visit: F)
    where
        F: FnMut(Frozen<StableDiGraph<N, E, Ix>>, EdgeIndex<Ix>) -> bool,
    {
        self.graph.retain_edges(visit);

        self.recount();
    }

    pub fn edge_weights(&self) -> impl Iterator<Item = &E> {
        self.graph.edge_weights()
    }

    pub fn node_references(&self) -> NodeReferences<'_, N, Ix> {
        self.graph.node_references()
    }

    pub fn edge_references(&self) -> EdgeReferences<'_, E, Ix> {
        self.graph.edge_references()
    }

    pub fn find_edge(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> Option<EdgeIndex<Ix>> {
        self.graph.find_edge(a, b)
    }
}

impl<'a, N, E, Ix: IndexType> visit::GraphBase for CountStableGraph<N, E, Ix> {
    type EdgeId = EdgeIndex<Ix>;
    type NodeId = NodeIndex<Ix>;
}

impl<'a, N, E: 'a, Ix> visit::IntoNodeIdentifiers for &'a CountStableGraph<N, E, Ix>
where
    Ix: IndexType,
{
    type NodeIdentifiers = NodeIndices<'a, N, Ix>;
    fn node_identifiers(self) -> Self::NodeIdentifiers {
        self.graph.node_identifiers()
    }
}
impl<N, E, Ix: IndexType> Index<EdgeIndex<Ix>> for CountStableGraph<N, E, Ix> {
    type Output = E;

    fn index(&self, index: EdgeIndex<Ix>) -> &Self::Output {
        &self.graph[index]
    }
}

impl<N, E, Ix: IndexType> Index<NodeIndex<Ix>> for CountStableGraph<N, E, Ix> {
    type Output = N;

    fn index(&self, index: NodeIndex<Ix>) -> &Self::Output {
        &self.graph[index]
    }
}

impl<N, E, Ix> visit::Data for CountStableGraph<N, E, Ix>
where
    Ix: IndexType,
{
    type NodeWeight = N;
    type EdgeWeight = E;
}

impl<'a, N: 'a, E: 'a, Ix> visit::IntoEdgeReferences for &'a CountStableGraph<N, E, Ix>
where
    Ix: IndexType,
{
    type EdgeRef = EdgeReference<'a, E, Ix>;
    type EdgeReferences = EdgeReferences<'a, E, Ix>;

    /// Create an iterator over all edges in the graph, in indexed order.
    ///
    /// Iterator element type is `EdgeReference<E, Ix>`.
    fn edge_references(self) -> Self::EdgeReferences {
        self.graph.edge_references()
    }
}
