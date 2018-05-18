#![windows_subsystem = "windows"]
#![feature(nll)]

extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
extern crate crossbeam_channel;
extern crate failure;
extern crate packt_core;
extern crate tokio;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_process;

mod view;

fn main() {
    relm::run::<view::Win>(()).unwrap();
}
