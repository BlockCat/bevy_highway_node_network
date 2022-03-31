#![feature(map_try_insert)]
pub use directed_network::*;
pub use neighbourhood::*;
mod directed_network;
mod highway;
mod neighbourhood;

pub use highway::phase_1;

#[cfg(test)]
pub(crate) mod tests;
