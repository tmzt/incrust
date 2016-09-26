#![crate_name="incrust_plugin"]
#![crate_type="dylib"]
#![feature(quote, plugin_registrar, rustc_private)]

#[macro_use]
extern crate log;

extern crate syntax;
extern crate rustc;
extern crate rustc_plugin;

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

mod codegen;
mod node;
mod output_actions;

use codegen::IntoWriteStmt;
use output_actions::{OutputAction, IntoOutputActions};
use node::{Element, TemplateExpr, TemplateNode, parse_node, parse_contents};


#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("emit_rust_template", emit_rust_template);
    reg.register_macro("emit_rust_and_js_template", emit_rust_and_js_template);
}

trait IntoViewItem {
    fn into_view_item<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Item>;
}

trait IntoBlock {
    fn into_block<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Block>;
}

struct View {
    name: String,
    span: Span,
    nodes: Vec<TemplateNode>
}

impl IntoViewItem for View {
    fn into_view_item<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Item> {
        create_view_item(ecx, self.span, &self)
    }
}


impl IntoOutputActions for View {
    fn into_output_actions<'cx>(&self, ecx: &'cx ExtCtxt) -> Vec<OutputAction> {
        let name = &self.name;
        let nodes = &self.nodes;

        let w_ident = ecx.ident_of("out");
        let mut stmts = Vec::new();

        let out_stmt = quote_stmt!(ecx, let mut $w_ident = String::new()).unwrap();
        stmts.push(out_stmt);

        let output_actions: Vec<OutputAction> = nodes.iter()
            .flat_map(|node| node.into_output_actions(ecx))
            .collect();

        output_actions
    }
}

impl IntoBlock for View {
    fn into_block<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Block> {
        let name = &self.name;
        let nodes = &self.nodes;

        let w_ident = ecx.ident_of("out");
        let mut stmts = Vec::new();

        let out_stmt = quote_stmt!(ecx, let mut $w_ident = String::new()).unwrap();
        stmts.push(out_stmt);

        let write_stmts: Vec<ast::Stmt> = nodes.iter()
            .flat_map(|node| node.into_output_actions(ecx))
            .map(|output_action| output_action.into_write_stmt(ecx, w_ident))
            .collect();
        stmts.extend(write_stmts);

        // Return rendered string for now
        stmts.push(quote_stmt!(ecx, $w_ident).unwrap());

        ecx.block(self.span, stmts)
    }
}

fn parse_view<'cx, 'a>(ecx: &'cx ExtCtxt, parser: &mut Parser<'a>, span: Span) -> PResult<'a, View> {
    let view_token = parser.parse_ident().unwrap();
    let view_name = parser.parse_ident().unwrap();

    try!(parser.expect(&token::OpenDelim(token::Bracket)));

    let nodes = try!(parse_contents(ecx, parser, span));

    Ok(View { name: view_name.name.to_string(), span: span, nodes: nodes })
}

fn create_view_item<'cx>(ecx: &'cx ExtCtxt, span: Span, view: &View) -> P<ast::Item> {
    let name = ecx.ident_of(&format!("rusttemplate_view_{}", view.name));
    let block = view.into_block(ecx);

    let inputs = vec![];
    let ret_ty = quote_ty!(ecx, String);
    ecx.item_fn(DUMMY_SP, name, inputs, ret_ty, block)
}

fn create_template_block<'cx>(ecx: &'cx ExtCtxt, span: Span, views: Vec<View>) -> Box<MacResult + 'cx> {
    let view_item_stmts: Vec<ast::Stmt> = views.iter()
        .map(|view| view.into_view_item(ecx))
        .map(|item| ecx.stmt_item(span, item))
    .collect();

    let mut stmts = Vec::new();
    stmts.extend(view_item_stmts);

    let name = ecx.ident_of("rusttemplate_view_root");
    let args = vec![];
    let call_expr = ecx.expr_call_ident(span, name, args);
    stmts.push(ecx.stmt_expr(call_expr));

    let block = ecx.block(span, stmts);

    MacEager::expr(ecx.expr_block(block))
}

fn parse_js_out_var<'cx, 'a>(ecx: &'cx mut ExtCtxt, parser: &mut Parser<'a>) -> PResult<'a, P<ast::Expr>> {
    // Read js variable expression
    let js_ident = try!(parser.parse_expr());
    // Consume ,
    try!(parser.expect(&token::Comma));

    Ok(js_ident)
}

fn construct_js_function(view_name: String, output_actions: Vec<OutputAction>) -> &'static str {
    ""
}

fn parse_emit_rust_template<'cx, 'a>(ecx: &'cx mut ExtCtxt, parser: &mut Parser<'a>, js_ident: Option<P<ast::Expr>>) -> PResult<'a, Box<MacResult + 'cx>> {
    let view = try!(parse_view(ecx, parser, DUMMY_SP));
    let views = vec![view];

    Ok(create_template_block(ecx, DUMMY_SP, views))
}

fn parse_emit_js_and_rust_template<'cx, 'a>(ecx: &'cx mut ExtCtxt, parser: &mut Parser<'a>) -> PResult<'a, Box<MacResult + 'cx>> {
    let js_ident = try!(parse_js_out_var(ecx, parser));
    parse_emit_rust_template(ecx, parser, Some(js_ident))
}

fn emit_rust_template<'cx>(
        ecx: &'cx mut ExtCtxt,
        span: Span,
        tts: &[TokenTree]) -> Box<MacResult + 'cx> {

    let mut parser = ecx.new_parser_from_tts(tts);
    match parse_emit_rust_template(ecx, &mut parser, None) {
        Err(mut err) => { err.emit(); return DummyResult::expr(DUMMY_SP); },
        Ok(result) => result
    }
}

fn emit_rust_and_js_template<'cx>(
        ecx: &'cx mut ExtCtxt,
        span: Span,
        tts: &[TokenTree]) -> Box<MacResult + 'cx> {

    let mut parser = ecx.new_parser_from_tts(tts);
    match parse_emit_js_and_rust_template(ecx, &mut parser) {
        Err(mut err) => { err.emit(); return DummyResult::expr(DUMMY_SP); },
        Ok(result) => result
    }
}