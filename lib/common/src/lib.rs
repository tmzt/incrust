#![crate_name="incrust_common"]
#![feature(quote, rustc_private)]

#[macro_use]
extern crate log;

extern crate syntax;
extern crate syntax_pos;
extern crate rustc;
extern crate rustc_plugin;
extern crate itertools;

use rustc_plugin::Registry;

use std::fmt::Write;

use syntax::abi::Abi;
use syntax::ast::{self, DUMMY_NODE_ID};

use syntax::codemap::{Span, Spanned, dummy_spanned, respan, spanned, DUMMY_SP};
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult};
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::ToTokens;
use syntax::print::pprust::{token_to_string, tts_to_string};
use syntax::tokenstream::TokenTree;
use syntax::util::small_vector::SmallVector;
use syntax::parse::{token, PResult};
use syntax::parse::common::SeqSep;
use syntax::parse::parser::Parser;
use syntax::ptr::P;

pub mod codegen;
pub mod output_actions;
pub mod simple_expr;
pub mod object_expr;
pub mod common_write;
pub mod js_write;
pub mod value;
pub mod nodes;


/*
pub fn tts_to_template<'cx, 'a>(ecx: &'cx mut ExtCtxt, mut parser: &mut Parser<'a>, tts: &[TokenTree]) -> PResult<'a, Template> {
    let template = try!(nodes::template_node::parse::parse_template(ecx, &mut parser, DUMMY_SP));
    Ok(template)
}
*/
