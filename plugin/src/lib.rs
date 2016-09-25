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


#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("emit_rust_template", emit_rust_template);
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
    tokens: Vec<TokenTree>
}

impl IntoViewItem for View {
    fn into_view_item<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Item> {
        create_view_item(ecx, self.span, &self)
    }
}

struct Element {
    element_type: String,
    span: Span,
    tokens: Vec<TokenTree>
}

impl IntoBlock for Element {
    fn into_block<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Block> {
        let element_type = &self.element_type;
        let stmt = quote_stmt!(ecx,
            {
                println!("Opening and closing [{}] element", $element_type);
                format!("<{}></{}>", $element_type, $element_type)
            }
        ).unwrap();

        let stmts = vec![ stmt ];
        ecx.block(self.span, stmts)
    }
}

fn parse_element<'a>(parser: &mut Parser<'a>, span: Span) -> PResult<'a, Element> {
    let element_type = parser.parse_ident().unwrap();
    try!(parser.expect(&token::OpenDelim(token::Bracket)));

    let tokens = parser.parse_seq_to_end(
        &token::CloseDelim(token::Bracket),
        SeqSep::none(),
        |parser| parser.parse_token_tree())
        .unwrap();

    Ok(Element { element_type: element_type.name.to_string(), span: span, tokens: tokens })
}

fn create_view_item<'cx>(ecx: &'cx ExtCtxt, span: Span, view: &View) -> P<ast::Item> {
    let name = ecx.ident_of(&format!("rusttemplate_view_{}", view.name));

    let mut parser = ecx.new_parser_from_tts(&view.tokens);
    let element = parse_element(&mut parser, span).unwrap();
    let block = element.into_block(ecx);

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

    /*
    let block = ecx.block(span, vec![
            ecx.stmt_item(span, item),
            ecx.stmt_expr(call_expr)]);
    */
    let block = ecx.block(span, stmts);

    MacEager::expr(ecx.expr_block(block))
}

fn parse_view_body<'a>(parser: &mut Parser<'a>) -> PResult<'a, Vec<TokenTree>> {
    try!(parser.expect(&token::OpenDelim(token::Bracket)));

    parser.parse_seq_to_end(
        &token::CloseDelim(token::Bracket),
        SeqSep::none(),
        |parser| parser.parse_token_tree())
}

fn parse_single_view<'a>(parser: &mut Parser<'a>, span: Span) -> PResult<'a, View> {
    let view_token = parser.parse_ident().unwrap();
    let view_name = parser.parse_ident().unwrap();

    let view_tokens = parse_view_body(parser).unwrap();

    Ok(View { name: view_name.name.to_string(), span: span, tokens: view_tokens })
}

fn parse_template<'cx, 'a>(ecx: &'cx ExtCtxt, parser: &mut Parser<'a>) -> Box<MacResult + 'cx> {
    let view = parse_single_view(parser, DUMMY_SP).unwrap();
    let views = vec![view];

    create_template_block(ecx, DUMMY_SP, views)
}

fn emit_rust_template<'cx>(
        ecx: &'cx mut ExtCtxt,
        span: Span,
        tts: &[TokenTree]) -> Box<MacResult + 'cx> {

    let mut parser = ecx.new_parser_from_tts(tts);
    parse_template(ecx, &mut parser)


    /*
    let mut i = 0;
    loop {
        match (tts.get(i), tts.get(i+1), tts.get(i+2)) {
            (Some(&TokenTree::Token(_, token::Ident(element_type))), _, _) => {
                ecx.span_warn(span, &format!("Outputing elementOpen for {}", &element_type.to_string()));

                let mut parser = ecx.new_parser_from_tts(tts);
                return parse_template(ecx, &mut parser);
            },
            (Some(_), _, _) => break,
            (None, _, _) => break
        }
    }
    */

    //MacEager::stmts(SmallVector::many(result))
    //MacEager::items(SmallVector::many(items))
    //DummyResult::any(span)
}