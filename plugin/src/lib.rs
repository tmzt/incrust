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

extern crate itertools;
use itertools::Itertools;

extern crate incrust_common;

use incrust_common::codegen::create_template_block;
use incrust_common::node::{Element, TemplateExpr, TemplateNode, parse_node, parse_contents};
use incrust_common::output_actions::{OutputAction, IntoOutputActions};
use incrust_common::view::parse_view;


#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("parse_template", parse_template);
    reg.register_macro("emit_rust_template", emit_rust_template);
    reg.register_macro("emit_rust_and_js_template", emit_rust_and_js_template);
}

fn parse_template<'cx, 'a>(ecx: &'cx mut ExtCtxt, span: Span, tts: &[TokenTree])
                                        -> Box<MacResult + 'cx> {
    //let tokens: Vec<TokenTree> = tts.into();
    let mut parser = ecx.new_parser_from_tts(tts);
    match parse_view(ecx, &mut parser, span) {
        Ok(view) => {
            // Convert view to output actions
            //let output_actions = view.into_output_actions(ecx);
            //ecx.span_warn(span, &format!("output_actions: {:?}", output_actions));

            //ecx.span_warn(span, format!("output_actions: {:?}", output_actions.iter().map(|a| format!("{:?}", a)).join(", ".into()).collect()));

            /*
            let quoted = quote_stmt!(ecx, {
                extern crate incrust_common;
                use incrust_common::output_actions::OutputAction;
                
                let actions = vec![OutputAction::WriteOpen(String::new("h1"))];
                ""
            }).unwrap();
            */


            /*
            let stmt = quote_stmt!(ecx, {
                extern crate incrust_common;
                use incrust_common::view::View;
                use incrust_common::Template;

                let view = View::from_output_actions($quoted)
                Template::from_views(vec![view])
            }).unwrap();
            */
            //let block = ecx.block(span, vec![quoted]);
            //MacEager::expr(ecx.expr_block(block))
            //DummyResult::expr(span)

            //let data = vec!["fake data", "extra"];
            let output_actions = vec![
                OutputAction::WriteOpen("h1".to_owned()),
                OutputAction::WriteClose("h1".to_owned())
            ];
            ecx.span_warn(span, &format!("output_actions: {:?}", output_actions));

            let s: Vec<TokenTree> = output_actions.iter()
                .map(|el| el.to_tokens(ecx))
                .intersperse(vec![TokenTree::Token(DUMMY_SP, token::Comma)])
                .flat_map(|el| el.to_tokens(ecx))
                .collect();

            let name = view.name();
            let s_name = quote_expr!(ecx, $name.to_owned());
            let stmt = quote_stmt!(ecx, {{
                extern crate incrust_common;
                use incrust_common::compiled_view::CompiledView;
                use incrust_common::output_actions::OutputAction;
                let actions = vec![$s];
                CompiledView::from_output_actions($s_name, actions)
            }}).unwrap();
            let block = ecx.block(span, vec![stmt]);
            MacEager::expr(ecx.expr_block(block))
        },

        Err(mut err) => {
            err.emit();
            DummyResult::expr(span)
        }
    }
}

fn parse_emit_rust_template<'cx, 'a>(ecx: &'cx mut ExtCtxt,
                                     parser: &mut Parser<'a>,
                                     js_ident: Option<P<ast::Expr>>)
                                     -> PResult<'a, Box<MacResult + 'cx>> {
    let view = try!(parse_view(ecx, parser, DUMMY_SP));
    let views = vec![view];

    Ok(create_template_block(ecx, DUMMY_SP, views))
}

pub fn parse_js_out_var<'cx, 'a>(ecx: &'cx mut ExtCtxt,
                             parser: &mut Parser<'a>)
                             -> PResult<'a, P<ast::Expr>> {
    // Read js variable expression
    let js_ident = try!(parser.parse_expr());
    // Consume ,
    try!(parser.expect(&token::Comma));

    Ok(js_ident)
}

fn parse_emit_js_and_rust_template<'cx, 'a>(ecx: &'cx mut ExtCtxt,
                                            parser: &mut Parser<'a>)
                                            -> PResult<'a, Box<MacResult + 'cx>> {
    let js_ident = try!(parse_js_out_var(ecx, parser));
    parse_emit_rust_template(ecx, parser, Some(js_ident))
}

fn emit_rust_template<'cx>(ecx: &'cx mut ExtCtxt,
                           span: Span,
                           tts: &[TokenTree])
                           -> Box<MacResult + 'cx> {

    let mut parser = ecx.new_parser_from_tts(tts);
    match parse_emit_rust_template(ecx, &mut parser, None) {
        Err(mut err) => {
            err.emit();
            return DummyResult::expr(DUMMY_SP);
        }
        Ok(result) => result,
    }
}

fn emit_rust_and_js_template<'cx>(ecx: &'cx mut ExtCtxt,
                                  span: Span,
                                  tts: &[TokenTree])
                                  -> Box<MacResult + 'cx> {

    let mut parser = ecx.new_parser_from_tts(tts);
    match parse_emit_js_and_rust_template(ecx, &mut parser) {
        Err(mut err) => {
            err.emit();
            return DummyResult::expr(DUMMY_SP);
        }
        Ok(result) => result,
    }
}
