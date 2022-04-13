use crate::{EdgeId, NodeId, ShortcutState};

pub trait NetworkData: Send + Sync + Default {
    type NodeData;
    type EdgeData;
    fn node_data(&self, node: NodeId) -> &Self::NodeData;
    fn edge_data(&self, edge: EdgeId) -> &Self::EdgeData;
    fn edge_road_id(&self, edge: EdgeId) -> ShortcutState<usize>;

    fn with_size(node_size: usize, edge_size: usize) -> Self;
    fn add_node(&mut self, node: NodeId, data: Self::NodeData);
    fn add_edge(&mut self, edge: EdgeId, data: Self::EdgeData, road_id: ShortcutState<usize>);
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

    fn edge_road_id(&self, edge: EdgeId) -> ShortcutState<usize> {
        ShortcutState::Single(edge.0 as usize)
    }

    fn add_node(&mut self, _: NodeId, _: Self::NodeData) {}

    fn add_edge(&mut self, edge: EdgeId, data: Self::EdgeData, road_id: ShortcutState<usize>) {}

    fn with_size(_: usize, _: usize) -> Self {
        ()
    }
}
