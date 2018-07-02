#[macro_use]
extern crate failure;
extern crate rand;
extern crate crossbeam_channel;
extern crate tokio;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_process;

pub mod geometry;
pub mod problem;
pub mod solution;
pub mod runner;
