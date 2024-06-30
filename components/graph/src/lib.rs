//! A library for working with networks.
//! This lib provides a way to create and manipulate networks.
//!
#![feature(map_try_insert)]
#![feature(is_sorted)]

use std::ops::Deref;

pub use directed_graph::*;
pub use neighbourhood::*;

/// Tja
pub mod directed_graph;

pub mod highway_network;
pub mod neighbourhood;

use serde::Deserialize;
use serde::Serialize;

mod dijkstra_iterator;

pub struct Forward;
pub struct Backward;

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct F32(pub f32);

impl Ord for F32 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Eq for F32 {}

#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct NodeId(pub u32);

impl From<usize> for NodeId {
    fn from(id: usize) -> Self {
        Self(id as u32)
    }
}

impl Deref for NodeId {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct EdgeId(pub u32);

impl From<usize> for EdgeId {
    fn from(id: usize) -> Self {
        Self(id as u32)
    }
}

impl From<u32> for EdgeId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl Deref for EdgeId {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &(self.0)
    }
}

impl From<u32> for NodeId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ShortcutState<T> {
    Single(T),
    Shortcut(Vec<T>),
}

impl<T> From<ShortcutState<T>> for Vec<T> {
    fn from(s: ShortcutState<T>) -> Self {
        match s {
            ShortcutState::Single(a) => vec![a],
            ShortcutState::Shortcut(a) => a,
        }
    }
}

// #[cfg(test)]
// pub(crate) mod tests;

#[macro_export]
macro_rules! create_network {
    ($s:literal..$e:literal, $($a:literal => $b:literal; $c: expr),+) => {
    {
        use $crate::builder::DefaultEdgeBuilder;
        use $crate::builder::DirectedNetworkBuilder;
        let mut builder = DirectedNetworkBuilder::<usize, DefaultEdgeBuilder>::new();

        for x in $s..=$e {
            builder.add_node(x);
        }

        $({
            let source = builder.add_node($a);
            let target = builder.add_node($b);

            builder.add_edge(DefaultEdgeBuilder::forward(source, target, 0, $c));

        })+

        builder.build::<()>()
    }
    };
}
