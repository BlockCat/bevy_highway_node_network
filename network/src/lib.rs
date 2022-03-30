#![feature(map_try_insert)]
pub use neighbourhood::*;
pub use network::*;
mod highway;
mod neighbourhood;
mod network;

pub use highway::phase_1;
