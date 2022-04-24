
use network::{highway_network::HighwayNetwork, NodeId, iterators::F32};

struct HeapEntry {
    distance: f32,
    level: u8,
    gap: f32,

}

pub fn shortest_path(source: NodeId, target: NodeId, network: &HighwayNetwork) {
    unimplemented!()
}

struct HighwayDijkstraIterator {
    
}