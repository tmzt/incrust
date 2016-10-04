
#![feature(plugin)]
#![plugin(incrust_plugin)]

extern crate incrust_common;

#[macro_use]
extern crate incrust_macros;

pub mod models;
pub mod render;