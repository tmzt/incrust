#![feature(concat_idents)]

#![feature(plugin)]
#![plugin(incrust_plugin)]

#[macro_use]
extern crate nickel;

#[macro_use]
extern crate incrust_macros;

mod render;

use nickel::{ Nickel, HttpRouter, StaticFilesHandler };
use render::render;

fn main() {
    let mut server = Nickel::new();
    server.utilize(StaticFilesHandler::new("./public"));
    server.get("**", middleware!(render()));

    server.listen("127.0.0.1:6767");
}