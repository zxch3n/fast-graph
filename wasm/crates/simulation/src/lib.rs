#![feature(ptr_internals)]
#![feature(ptr_metadata)]
extern crate core;

pub mod data;
pub mod force;
mod simulation;
pub use simulation::Simulation;
