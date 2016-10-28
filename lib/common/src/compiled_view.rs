
use std::fmt;

use syntax::ast;
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult};
use syntax::ext::build::AstBuilder;
use syntax::codemap::{Span, Spanned, dummy_spanned, respan, spanned, DUMMY_SP};
use syntax::parse::{token, PResult};
use syntax::parse::parser::Parser;
use syntax::ptr::P;

use itertools::Itertools;

use node::{Element, TemplateExpr, TemplateNode, parse_node, parse_contents};
use codegen::{IntoWriteStmt, IntoViewItem};
use jsgen::{IntoJsFunction, IntoJsOutputCall};
use output_actions::{OutputAction, IntoOutputActions};
use IntoBlock;


#[derive(Clone, Debug)]
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

    pub fn output_actions(&self) -> &[OutputAction] {
        self.output_actions.as_slice()
    }
}

fn create_view_item<'cx>(ecx: &'cx ExtCtxt, compiled_view: &CompiledView) -> P<ast::Item> {
    let name = ecx.ident_of(&format!("rusttemplate_view_{}", compiled_view.name));
    let block = compiled_view.into_block(ecx);

    let inputs = vec![];
    let ret_ty = quote_ty!(ecx, String);
    ecx.item_fn(DUMMY_SP, name, inputs, ret_ty, block)
}

impl IntoViewItem for CompiledView {
    fn into_view_item<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Item> {
        create_view_item(ecx, &self)
    }
}

/*
impl IntoOutputActions for CompiledView {
    fn into_output_actions<'cx>(&self, ecx: &'cx ExtCtxt) -> Vec<OutputAction> {
        let name = &self.name;
        let output_actions: Vec<OutputAction> = self.output_actions.iter().collect();

        let w_ident = ecx.ident_of("out");
        let mut stmts = Vec::new();

        let out_stmt = quote_stmt!(ecx, let mut $w_ident = String::new()).unwrap();
        stmts.push(out_stmt);

        output_actions
    }
}
*/

impl IntoBlock for CompiledView {
    fn into_block<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Block> {
        let name = &self.name;
        let output_actions = &self.output_actions;

        let w_ident = ecx.ident_of("out");
        let mut stmts = Vec::new();

        let out_stmt = quote_stmt!(ecx, let mut $w_ident = String::new()).unwrap();
        stmts.push(out_stmt);

        let write_stmts: Vec<ast::Stmt> = output_actions.iter()
            .map(|output_action| output_action.into_write_stmt(ecx, w_ident))
            .collect();
        stmts.extend(write_stmts);

        // Return rendered string for now
        stmts.push(quote_stmt!(ecx, $w_ident).unwrap());

        ecx.block(DUMMY_SP, stmts)
    }
}

impl IntoJsFunction for CompiledView {
    fn into_js_function<'cx>(&self, ecx: &'cx ExtCtxt) -> String {
        let name = &self.name;
        let output_actions = &self.output_actions;

        let js_stmts: Vec<String> = output_actions.iter()
            .map(|output_action| output_action.into_js_output_call())
            .intersperse("; ".into())
            .collect();
        
        let js_body: String = js_stmts.join(" ");
        let js = format!("function render_view_{}(data) {{ {} }}", name, js_body);

        js
    }
}
