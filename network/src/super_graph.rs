use std::{
    iter::Enumerate,
    marker::PhantomData,
    ops::{Index, Range},
    slice,
};

use petgraph::{
    stable_graph::{DefaultIx, EdgeIndex, IndexType, NodeIndex, StableDiGraph},
    visit::{self, EdgeRef, IntoEdgesDirected, IntoNeighborsDirected},
    Directed,
    Direction::{self, Incoming, Outgoing},
};
use serde::{Deserialize, Serialize};

/// The graph's node type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node<N, Ix: IndexType> {
    /// Associated node data
    pub weight: N,
    /// Next outgoing edge
    /// Next incoming edge, starts at first bothways edge
    /// Final outgoing edge (exclusive) also start of first incoming only edge
    pub next: [EdgeIndex<Ix>; 3],
}

impl<N, Ix: IndexType> Node<N, Ix> {
    pub fn next_edge(&self, dir: Direction) -> EdgeIndex<Ix> {
        self.next[dir.index()]
    }
}

/// The graph's edge type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge<E, Ix: IndexType> {
    /// Associated edge data.
    pub weight: E,
    /// Me -> Other
    node: [NodeIndex<Ix>; 2],
}

impl<E, Ix: IndexType> Edge<E, Ix> {
    /// Comes from
    pub fn source(&self) -> NodeIndex<Ix> {
        self.node[0]
    }
    /// Goes to
    pub fn target(&self) -> NodeIndex<Ix> {
        self.node[1]
    }
}

/// Simple graph,
/// Nodes point to the first
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperGraph<N, E, Ix: IndexType = DefaultIx> {
    nodes: Vec<Node<N, Ix>>,
    /// per node: Forward|Both|Backward
    edges: Vec<Edge<E, Ix>>,
}

impl<N, E, Ix: IndexType> Default for SuperGraph<N, E, Ix> {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
            edges: Default::default(),
        }
    }
}

impl<N, E, Ix: IndexType> SuperGraph<N, E, Ix> {
    pub fn with_capacity(nodes: usize, edges: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(nodes),
            edges: Vec::with_capacity(edges),
        }
    }
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
    #[inline]
    pub fn is_directed(&self) -> bool {
        true
    }

    pub fn edge_reference(&self, e: EdgeIndex<Ix>) -> EdgeReference<'_, E, Ix> {
        EdgeReference {
            index: e,
            node: self.edges[e.index()].node.clone(),
            weight: &self.edges[e.index()].weight,
        }
    }

    pub fn node_weight(&self, a: NodeIndex<Ix>) -> Option<&N> {
        self.nodes.get(a.index()).map(|n| &n.weight)
    }
    pub fn node_weight_mut(&mut self, a: NodeIndex<Ix>) -> Option<&mut N> {
        self.nodes.get_mut(a.index()).map(|n| &mut n.weight)
    }

    pub fn edge_weight(&self, a: EdgeIndex<Ix>) -> Option<&E> {
        self.edges.get(a.index()).map(|e| &e.weight)
    }
    pub fn edge_weight_mut(&mut self, a: EdgeIndex<Ix>) -> Option<&mut E> {
        self.edges.get_mut(a.index()).map(|e| &mut e.weight)
    }
    pub fn edge_endpoints(&self, a: EdgeIndex<Ix>) -> Option<(NodeIndex<Ix>, NodeIndex<Ix>)> {
        self.edges.get(a.index()).map(|e| (e.source(), e.target()))
    }

    pub fn neighbors_undirected(&self, a: NodeIndex<Ix>) -> Neighbors<E, Ix> {
        let node = self.nodes[a.index()].next[0];
        let next_node = self
            .nodes
            .get(a.index())
            .map(|x| x.next[0])
            .unwrap_or(EdgeIndex::new(self.edges.len()));

        Neighbors {
            range: node.index()..next_node.index(),
            edges: &self.edges,
        }
    }
}

impl<N, E, Ix: IndexType> SuperGraph<N, E, Ix> {}

#[derive(Debug, Clone)]
pub struct Neighbors<'a, E: 'a, Ix: 'a + IndexType> {
    range: Range<usize>,
    edges: &'a [Edge<E, Ix>],
}

impl<'a, E, Ix: IndexType> Iterator for Neighbors<'a, E, Ix> {
    type Item = NodeIndex<Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().map(|i| self.edges[i].target())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.range.len()
    }
}

impl<N, E, Ix: IndexType> visit::GraphBase for SuperGraph<N, E, Ix> {
    type EdgeId = EdgeIndex<Ix>;
    type NodeId = NodeIndex<Ix>;
}

impl<N, E, Ix: IndexType> visit::GraphProp for SuperGraph<N, E, Ix> {
    type EdgeType = Directed;
}

impl<N, E, Ix: IndexType> visit::Data for SuperGraph<N, E, Ix> {
    type NodeWeight = N;
    type EdgeWeight = E;
}

impl<'a, N, E, Ix: IndexType> visit::IntoNeighbors for &'a SuperGraph<N, E, Ix> {
    type Neighbors = Neighbors<'a, E, Ix>;

    fn neighbors(self, a: Self::NodeId) -> Self::Neighbors {
        self.neighbors_directed(a, Outgoing)
    }
}

impl<'a, N, E, Ix: IndexType> visit::IntoNeighborsDirected for &'a SuperGraph<N, E, Ix> {
    type NeighborsDirected = Neighbors<'a, E, Ix>;

    fn neighbors_directed(self, a: NodeIndex<Ix>, dir: Direction) -> Neighbors<'a, E, Ix> {
        let node = &self.nodes[a.index()];
        let range = match dir {
            Outgoing => node.next[0].index()..node.next[2].index(),
            Incoming => {
                node.next[1].index()
                    ..self
                        .nodes
                        .get(a.index() + 1)
                        .map(|n| n.next[0].index())
                        .unwrap_or(self.edge_count())
            }
        };

        Neighbors {
            range,
            edges: &self.edges,
        }
    }
}

impl<'a, N: 'a, E: 'a, Ix: IndexType> visit::IntoEdgeReferences for &'a SuperGraph<N, E, Ix> {
    type EdgeRef = EdgeReference<'a, E, Ix>;
    type EdgeReferences = EdgeReferences<'a, E, Ix>;

    fn edge_references(self) -> Self::EdgeReferences {
        EdgeReferences {
            iter: self.edges.iter().enumerate(),
        }
    }
}
impl<'a, N, E, Ix: IndexType> visit::IntoEdges for &'a SuperGraph<N, E, Ix> {
    type Edges = Edges<'a, E, Ix>;

    fn edges(self, a: Self::NodeId) -> Self::Edges {
        self.edges_directed(a, Direction::Outgoing)
    }
}

impl<'a, N, E, Ix: IndexType> visit::IntoEdgesDirected for &'a SuperGraph<N, E, Ix> {
    type EdgesDirected = Edges<'a, E, Ix>;

    fn edges_directed(self, _a: Self::NodeId, _dir: Direction) -> Self::EdgesDirected {
        todo!()
    }
}

impl<'a, N, E: 'a, Ix: IndexType> visit::IntoNodeIdentifiers for &'a SuperGraph<N, E, Ix> {
    type NodeIdentifiers = NodeIndices<Ix>;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        NodeIndices {
            r: 0..self.nodes.len(),
            ty: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Edges<'a, E: 'a, Ix: 'a + IndexType> {
    range: Range<usize>,
    edges: &'a [Edge<E, Ix>],
}

impl<'a, E, Ix: IndexType> Iterator for Edges<'a, E, Ix> {
    type Item = EdgeReference<'a, E, Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().map(|i| {
            let edge = &self.edges[i];
            EdgeReference {
                index: EdgeIndex::new(i),
                weight: &edge.weight,
                node: edge.node.clone(),
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.range.len()
    }
}

#[derive(Debug)]
pub struct EdgeReference<'a, E: 'a, Ix> {
    index: EdgeIndex<Ix>,
    node: [NodeIndex<Ix>; 2],
    weight: &'a E,
}

impl<'a, E: 'a, Ix: IndexType> EdgeRef for EdgeReference<'a, E, Ix> {
    type NodeId = NodeIndex<Ix>;
    type EdgeId = EdgeIndex<Ix>;
    type Weight = E;

    fn source(&self) -> Self::NodeId {
        self.node[0]
    }

    fn target(&self) -> Self::NodeId {
        self.node[1]
    }

    fn weight(&self) -> &Self::Weight {
        self.weight
    }

    fn id(&self) -> Self::EdgeId {
        self.index
    }
}

impl<'a, E, Ix: IndexType> Clone for EdgeReference<'a, E, Ix> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, E, Ix: IndexType> Copy for EdgeReference<'a, E, Ix> {}

impl<'a, E, Ix: IndexType> PartialEq for EdgeReference<'a, E, Ix>
where
    E: PartialEq,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.index == rhs.index && self.weight == rhs.weight
    }
}

impl<'a, Ix, E> EdgeReference<'a, E, Ix>
where
    Ix: IndexType,
{
    /// Access the edge’s weight.
    ///
    /// **NOTE** that this method offers a longer lifetime
    /// than the trait (unfortunately they don't match yet).
    pub fn weight(&self) -> &'a E {
        self.weight
    }
}

pub struct NodeIndices<Ix> {
    r: Range<usize>,
    ty: PhantomData<Ix>,
}

impl<Ix: IndexType> Iterator for NodeIndices<Ix> {
    type Item = NodeIndex<Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        self.r.next().map(NodeIndex::new)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.r.size_hint()
    }
}

#[derive(Debug, Clone)]
pub struct EdgeReferences<'a, E: 'a, Ix: IndexType> {
    iter: Enumerate<slice::Iter<'a, Edge<E, Ix>>>,
}

impl<'a, E, Ix: IndexType> Iterator for EdgeReferences<'a, E, Ix> {
    type Item = EdgeReference<'a, E, Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(i, edge)| EdgeReference {
            index: EdgeIndex::new(i),
            node: edge.node.clone(),
            weight: &edge.weight,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.iter.count()
    }
}

/// Node indices are not invalidated
impl<N, E, Ix: IndexType> From<SuperGraph<N, E, Ix>> for StableDiGraph<N, E, Ix> {
    fn from(_value: SuperGraph<N, E, Ix>) -> Self {
        todo!()
    }
}

/// Node indices are invalidated
impl<N, E, Ix: IndexType> From<StableDiGraph<N, E, Ix>> for SuperGraph<N, E, Ix> {
    fn from(_value: StableDiGraph<N, E, Ix>) -> Self {
        todo!()
    }
}

impl<N, E, Ix: IndexType> Index<EdgeIndex<Ix>> for SuperGraph<N, E, Ix> {
    type Output = E;

    fn index(&self, index: EdgeIndex<Ix>) -> &Self::Output {
        &self.edges[index.index()].weight
    }
}

impl<N, E, Ix: IndexType> Index<NodeIndex<Ix>> for SuperGraph<N, E, Ix> {
    type Output = N;

    fn index(&self, index: NodeIndex<Ix>) -> &Self::Output {
        &self.nodes[index.index()].weight
    }
}