extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
extern crate packt_core;

mod view;

fn main() {
    relm::run::<view::Win>(()).unwrap();
}
