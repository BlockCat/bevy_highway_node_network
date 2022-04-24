use network::{EdgeId, NodeId};
use std::collections::{BinaryHeap, HashMap};

#[derive(Debug, PartialEq, PartialOrd)]
pub struct ParentEntry {
    pub parent: NodeId,
    pub parent_edge: EdgeId,
    pub parent_edge_distance: f32,
    pub active: bool,
}

#[derive(Debug, PartialEq)]
pub struct DijkstraNodeState {
    pub distance: f32,
    pub current: NodeId,
    pub parent: ParentEntry,
}

impl Ord for DijkstraNodeState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Eq for DijkstraNodeState {}

impl PartialOrd for DijkstraNodeState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.distance.partial_cmp(&other.distance) {
            Some(core::cmp::Ordering::Equal) => {}
            Some(core::cmp::Ordering::Greater) => return Some(core::cmp::Ordering::Less),
            Some(core::cmp::Ordering::Less) => return Some(core::cmp::Ordering::Greater),
            None => return None,
        }

        match self.current.partial_cmp(&other.current) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }

        self.parent.partial_cmp(&other.parent)
    }
}

#[derive(Debug)]
pub struct VisitedState {
    pub border_distance: f32,
    pub reference_distance: f32,
    pub distance: f32,
    pub parents: HashMap<NodeId, (Option<EdgeId>, f32)>,
}

pub struct HighwayNodeQueue {
    pub heap: BinaryHeap<DijkstraNodeState>,
    pub visited: HashMap<NodeId, VisitedState>,
    active: usize,
}

pub struct HighwayQueueEntry {
    pub state: DijkstraNodeState,
    pub parents: HashMap<NodeId, (Option<EdgeId>, f32)>,
    pub border_distance: f32,
    pub reference_distance: f32,
    pub parent_active: bool,
}

impl HighwayNodeQueue {
    pub fn new(heap_size: usize, visited_size: usize) -> Self {
        Self {
            heap: BinaryHeap::with_capacity(heap_size),
            visited: HashMap::with_capacity(visited_size),
            active: 0,
        }
    }
    pub fn push(&mut self, state: DijkstraNodeState) {
        if state.parent.active {
            self.active += 1;
        }
        self.heap.push(state);
    }

    pub fn pop(&mut self) -> Option<HighwayQueueEntry> {
        if !self.is_active() {
            return None;
        }
        while let Some(mut state) = self.heap.pop() {
            if state.parent.active {
                self.active -= 1;
            }
            if self.visited.contains_key(&state.current) {
                debug_assert!(state.distance > self.visited[&state.current].distance);
                continue;
            }

            let mut parent_border_distance = self.visited[&state.parent.parent].border_distance;
            let mut parent_reference_distance =
                self.visited[&state.parent.parent].reference_distance;
            let mut parents = HashMap::from([(
                state.parent.parent,
                (
                    Some(state.parent.parent_edge),
                    state.parent.parent_edge_distance,
                ),
            )]);

            let mut parent_active = state.parent.active;

            while let Some(peek) = self
                .heap
                .peek()
                .filter(|next| next.current == state.current && next.distance == state.distance)
            {
                if peek.parent.active {
                    self.active -= 1;
                }

                let visited_parent = &self.visited[&peek.parent.parent];

                parent_border_distance =
                    f32::max(parent_border_distance, visited_parent.border_distance);
                parent_reference_distance =
                    f32::max(parent_reference_distance, visited_parent.reference_distance);

                parents.insert(
                    peek.parent.parent,
                    (
                        Some(peek.parent.parent_edge),
                        peek.parent.parent_edge_distance,
                    ),
                );

                parent_active |= peek.parent.active;

                state = self.heap.pop().unwrap();
            }

            return Some(HighwayQueueEntry {
                state,
                border_distance: parent_border_distance,
                reference_distance: parent_reference_distance,
                parents,
                parent_active,
            });
        }

        None
    }

    pub fn visited(&mut self, node: NodeId, state: VisitedState) -> Option<VisitedState> {
        self.visited.insert(node, state)
    }

    pub fn is_active(&self) -> bool {
        debug_assert_eq!(
            self.active,
            self.heap.iter().filter(|x| x.parent.active).count()
        );
        self.active > 0
    }
}
