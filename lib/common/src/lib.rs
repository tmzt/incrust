#![crate_name="incrust_common"]
#![feature(quote, rustc_private)]

#[macro_use]
extern crate log;

extern crate syntax;
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
pub mod template_node;
pub mod view;
pub mod output_actions;
pub mod compiled_view;
pub mod simple_expr;
pub mod js_write;

use codegen::{IntoWriteStmt, create_template_block};
use output_actions::{OutputAction, IntoOutputActions};
use view::{View, parse_view};


// Represents a parsed template in incrust
pub struct Template {
    views: Vec<View>
}

impl Template {
    pub fn from_views(views: Vec<View>) -> Template {
        Template { views: views }
    }
}

trait IntoBlock {
    fn into_block<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Block>;
}

pub fn tts_to_template<'cx, 'a>(ecx: &'cx mut ExtCtxt, parser: &mut Parser<'a>, tts: &[TokenTree]) -> PResult<'a, Template> {
    
    let view = try!(parse_view(ecx, parser, DUMMY_SP));
    let views = vec![view];

    Ok(Template { views: views })
}
