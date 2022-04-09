#![feature(map_try_insert)]
pub use directed_network::*;
pub use neighbourhood::*;

mod directed_network;
mod highway;
mod neighbourhood;

pub use highway::intermediate_network;

pub use highway::calculate_layer;

#[cfg(test)]
pub(crate) mod tests;
