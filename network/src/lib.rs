#![feature(map_try_insert)]
#![feature(is_sorted)]

pub use directed_network::*;
pub use neighbourhood::*;

pub mod directed_network;
pub mod highway;
pub mod neighbourhood;

pub use highway::intermediate_network;

pub use highway::calculate_layer;

#[cfg(test)]
pub(crate) mod tests;
