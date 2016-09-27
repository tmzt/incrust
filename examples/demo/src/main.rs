#![feature(plugin)]
#![plugin(incrust_plugin)] extern crate incrust_plugin;


#[macro_use]
extern crate nickel;

#[macro_use]
extern crate incrust_macros;

extern crate incrust_common;

mod render;

use nickel::{ Nickel, HttpRouter, StaticFilesHandler };
use render::render;

fn main() {
    let mut server = Nickel::new();
    server.utilize(StaticFilesHandler::new("./public"));
    server.get("**", middleware!(render()));

    server.listen("127.0.0.1:6767");
}