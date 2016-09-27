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

use std::rc::Rc;

use syntax::abi::Abi;
use syntax::ast::{self, DUMMY_NODE_ID};

use syntax::codemap::{Span, Spanned, dummy_spanned, respan, spanned, DUMMY_SP};
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult, NormalTT, IdentTT, TTMacroExpander, Resolver};
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::ToTokens;
use syntax::ext::hygiene::Mark;
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

use incrust_common::codegen;
use incrust_common::node::{Element, TemplateExpr, TemplateNode, parse_node, parse_contents};
use incrust_common::output_actions::{OutputAction, IntoOutputActions};
use incrust_common::view::parse_view;
use incrust_common::compiled_view::CompiledView;


#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(token::intern("define_template"),
            syntax::ext::base::IdentTT(Box::new(expand_define_template), None, false));

    reg.register_macro("parse_template", parse_template);
    //reg.register_macro("emit_rust_compiled_view", emit_rust_compiled_view);
}

#[derive(Clone)]
struct NamedTemplateDecl {
    name: String,
    compiled_views: Vec<CompiledView>
}

impl TTMacroExpander for NamedTemplateDecl {
    fn expand<'cx>(&self, ecx: &'cx mut ExtCtxt, span: Span, tts: &[TokenTree]) -> Box<MacResult + 'cx> {
        let mut parser = ecx.new_parser_from_tts(tts);
        let w_ident = parser.parse_ident().unwrap();
        //let w_ident = ecx.ident_of("out");
        codegen::create_template_write_block(ecx, w_ident, &self.compiled_views)
    }
}

fn define_named_template<'cx>(ecx: &'cx mut ExtCtxt, name: String, compiled_views: Vec<CompiledView>) {
    let ext_name = &format!("emit_rust_view_{}", name);
    let ident = ecx.ident_of(ext_name);
    // TODO: This is changed to add_ext in b4906a (https://github.com/rust-lang/rust/commit/b4906a93)
    (*ecx.resolver).add_macro(Mark::root(), ident,
		Rc::new(NormalTT(Box::new(NamedTemplateDecl { name: name, compiled_views: compiled_views }), None, true)));
}

fn parse_template_into_compiled_view<'cx, 'a>(ecx: &'cx mut ExtCtxt, span: Span, parser: &mut Parser<'a>)
                                                                            -> PResult<'a, CompiledView> {
    //let tokens: Vec<TokenTree> = tts.into();
    let view = try!(parse_view(ecx, parser, span));

    let output_actions = vec![
        OutputAction::WriteOpen("h1".to_owned()),
        OutputAction::Write("testing".to_owned()),
        OutputAction::WriteClose("h1".to_owned())
    ];
    ecx.span_warn(span, &format!("output_actions: {:?}", output_actions));

    let name = String::from(view.name());
    Ok(CompiledView::from_output_actions(name, output_actions))
}

fn parse_template_into_compiled_view_result<'cx, 'a>(ecx: &'cx mut ExtCtxt, span: Span, parser: &mut Parser<'a>)
                                                                            -> PResult<'a, Box<MacResult + 'cx>> {
    //let tokens: Vec<TokenTree> = tts.into();
    let view = try!(parse_view(ecx, parser, span));

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
        Ok(MacEager::expr(ecx.expr_block(block)))
}

fn parse_template<'cx, 'a>(ecx: &'cx mut ExtCtxt, span: Span, tts: &[TokenTree])
                                        -> Box<MacResult + 'cx> {
    let mut parser = ecx.new_parser_from_tts(tts);
    //let tokens: Vec<TokenTree> = tts.into();
    match parse_template_into_compiled_view_result(ecx, span, &mut parser) {
        Ok(result) => {
            result
        },

        Err(mut err) => {
            err.emit();
            DummyResult::expr(span)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         
        }
    }                                                                       
}

fn expand_define_template<'cx>(ecx: &'cx mut ExtCtxt, span: Span, ident: ast::Ident, tts: Vec<TokenTree>) -> Box<MacResult + 'cx> {
    let name = ident.name.to_string();
    let mut parser = ecx.new_parser_from_tts(&tts);
    //let tokens: Vec<TokenTree> = tts.into();
    match parse_template_into_compiled_view(ecx, span, &mut parser) {
        Ok(compiled_view) => {
            let compiled_views = vec![compiled_view];
            define_named_template(ecx, name, compiled_views);
        },

        Err(mut err) => {
            err.emit();
        }
    }

    // Empty
    //DummyResult::items(span)
    MacEager::items(SmallVector::zero())
}

/*
fn expand_rust_compiled_view<'s>(cx: &'s mut ExtCtxt, sp: codemap::Span, ident: ast::Ident, tts: Vec<TokenTree>) -> Box<MacResult + 's> {
    let source = match parse_arg(cx, &tts) {
        Some(source) => source,
        None => return DummyResult::any(sp),
    };

    expand_peg(cx, sp, ident, &source)
}
*/

/*
fn parse_rust_compiled_view_emit<'cx, 'a>(ecx: &'cx mut ExtCtxt, parser: &Parser<'a>, span: Span, tts: &[TokenTree]) -> PResult<'a, Box<MacResult + 'cx>> {
    let ident = try!(parser.parse_ident());
    let 

}

fn emit_rust_compiled_view<'cx, 'a>(ecx: &'cx mut ExtCtxt, span: Span, tts: &[TokenTree])
                                        -> Box<MacResult + 'cx> {
    let mut parser = parse::new_parser_from_tts(tts);
    let compiled_views = vec![compiled_view];

    create_template_block(ecx, compiled_views)
}
*/