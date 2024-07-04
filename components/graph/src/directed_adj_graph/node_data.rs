use crate::{EdgeId, NodeId, ShortcutState};

#[derive(Debug)]
pub struct DefaultNetworkData<N, E> {
    nodes: Vec<Option<N>>,
    edges: Vec<Option<E>>,
    shortcuts: Vec<Option<ShortcutState<EdgeId>>>,
}

impl<N, E> Default for DefaultNetworkData<N, E> {
    fn default() -> Self {
        DefaultNetworkData {
            nodes: Vec::new(),
            edges: Vec::new(),
            shortcuts: Vec::new(),
        }
    }
}

pub trait NetworkData: Send + Sync + Default {
    type NodeData;
    type EdgeData;
    fn node_data(&self, node: NodeId) -> &Self::NodeData;
    fn edge_data(&self, edge: EdgeId) -> &Self::EdgeData;
    fn edge_road_id(&self, edge: EdgeId) -> ShortcutState<EdgeId>;

    fn with_size(node_size: usize, edge_size: usize) -> Self;
    fn add_node(&mut self, node: NodeId, data: Self::NodeData);
    fn add_edge(&mut self, edge: EdgeId, data: Self::EdgeData, road_id: ShortcutState<EdgeId>);
}

impl NetworkData for () {
    type NodeData = ();
    type EdgeData = ();

    fn node_data(&self, _: NodeId) -> &Self::NodeData {
        &()
    }

    fn edge_data(&self, _: EdgeId) -> &Self::EdgeData {
        &()
    }

    fn edge_road_id(&self, edge: EdgeId) -> ShortcutState<EdgeId> {
        ShortcutState::Single(edge)
    }

    fn add_node(&mut self, _: NodeId, _: Self::NodeData) {}

    fn add_edge(&mut self, _: EdgeId, _: Self::EdgeData, _: ShortcutState<EdgeId>) {}

    fn with_size(_: usize, _: usize) -> Self {
        ()
    }
}

impl<N, E> NetworkData for DefaultNetworkData<N, E>
where
    N: Send + Sync + Clone,
    E: Send + Sync + Clone,
{
    type NodeData = N;

    type EdgeData = E;

    fn node_data(&self, node: NodeId) -> &Self::NodeData {
        self.nodes[node.0 as usize].as_ref().unwrap()
    }

    fn edge_data(&self, edge: EdgeId) -> &Self::EdgeData {
        self.edges[edge.0 as usize].as_ref().unwrap()
    }

    fn edge_road_id(&self, edge: EdgeId) -> ShortcutState<EdgeId> {
        self.shortcuts[edge.0 as usize].clone().unwrap()
    }

    fn with_size(node_size: usize, edge_size: usize) -> Self {
        DefaultNetworkData {
            nodes: vec![None; node_size],
            edges: vec![None; edge_size],
            shortcuts: vec![None; edge_size],
        }
    }

    fn add_node(&mut self, node: NodeId, data: Self::NodeData) {
        self.nodes[node.0 as usize] = Some(data);
    }

    fn add_edge(&mut self, edge: EdgeId, data: Self::EdgeData, road_id: ShortcutState<EdgeId>) {
        self.edges[edge.0 as usize] = Some(data);
        self.shortcuts[edge.0 as usize] = Some(road_id);
    }
}
