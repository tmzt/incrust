
use std::fmt;

use syntax::ast;
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult};
use syntax::ext::build::AstBuilder;
use syntax::codemap::{Span, Spanned, dummy_spanned, respan, spanned, DUMMY_SP};
use syntax::parse::{token, PResult};
use syntax::parse::parser::Parser;
use syntax::ptr::P;

use node::{Element, TemplateExpr, TemplateNode, parse_node, parse_contents};
use codegen::{IntoWriteStmt, IntoViewItem};
use jsgen::{IntoJsFunction, IntoJsOutputCall};
use output_actions::{OutputAction, IntoOutputActions};
use IntoBlock;


#[derive(Debug)]
pub struct CompiledView {
    name: String,
    output_actions: Vec<OutputAction>
}

impl fmt::Display for CompiledView {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "view {} [{:?}]", &self.name, &self.output_actions)
    }
}

impl CompiledView {
    pub fn from_output_actions(name: String, output_actions: Vec<OutputAction>) -> CompiledView {
        CompiledView { name: name, output_actions: output_actions }
    }
}