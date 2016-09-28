#![feature(plugin)]
#![plugin(incrust_plugin)] extern crate incrust_plugin;


#[macro_use]
extern crate nickel;

#[macro_use]
extern crate incrust_macros;

extern crate incrust_common;

mod render;
mod templates;

use std::path::Path;

use nickel::{ Nickel, HttpRouter, StaticFilesHandler };
use render::render;

fn statics() -> StaticFilesHandler {
    const CARGO_MANIFEST_DIR: Option<&'static str> = option_env!("CARGO_MANIFEST_DIR");
    let root_dir = CARGO_MANIFEST_DIR.map(|s| Path::new(s))
        .expect("Must provide CARGO_MANIFEST_DIR as cargo does pointing to the folder containing public/");
    StaticFilesHandler::new(root_dir.join("public"))
}

fn main() {
    let mut server = Nickel::new();
    server.utilize(statics());
    server.get("**", middleware!(render()));

    server.listen("127.0.0.1:6767").unwrap();
}