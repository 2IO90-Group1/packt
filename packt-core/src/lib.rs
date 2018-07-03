#[macro_use]
extern crate failure;
extern crate crossbeam_channel;
extern crate rand;
extern crate serde;
extern crate tokio;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_process;
#[macro_use]
extern crate serde_derive;

pub mod geometry;
pub mod problem;
pub mod runner;
pub mod solution;
