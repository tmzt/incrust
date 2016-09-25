#![feature(concat_idents)]

#[macro_use]
extern crate nickel;

mod render;

use nickel::{Nickel, HttpRouter};
use render::render;

fn main() {
    let mut server = Nickel::new();
    server.get("**", middleware!(render()));
    server.listen("127.0.0.1:6767");
}
