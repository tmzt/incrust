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
use std::marker::PhantomData;

use syntax::abi::Abi;
use syntax::ast::{self, DUMMY_NODE_ID};

use syntax::codemap::{Span, Spanned, dummy_spanned, respan, spanned, DUMMY_SP};
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult, SyntaxExtension, NormalTT, IdentTT, TTMacroExpander, Resolver};
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
use incrust_common::jsgen::{IntoJsFunction, IntoJsOutputCall};
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

fn define_ext<'cx>(ecx: &'cx mut ExtCtxt, name: &str, ext: Rc<SyntaxExtension>) {
    let ident = ecx.ident_of(name);
    // TODO: This is changed to add_ext in b4906a (https://github.com/rust-lang/rust/commit/b4906a93)
    (*ecx.resolver).add_macro(Mark::root(), ident, ext);
}

macro_rules! define_named_template {
    ($ecx: ident, $name: ident, $ty:ident, $lang: expr, $views:expr) => ({
        let name = $name.name.to_string();
        let ext_name = &format!("emit_{}_view_{}", $lang, name);
        let ext = LangSyntaxExt::<$ty>::create_template(&name, $views.clone());
        define_ext($ecx, ext_name, Rc::new(NormalTT(Box::new(ext), None, true)));
    })
}

#[derive(Debug, Clone)]
struct NamedTemplateDecl {
    name: String,
    compiled_views: Vec<CompiledView>
}

enum Rust {}
enum Js {}

trait WriteBlockFactory {
    fn create_write_block<'cx>(&self, ecx: &'cx mut ExtCtxt, w_ident: ast::Ident) -> Box<MacResult + 'cx>;
}

trait TargetLangSyntaxExt {}

trait Lang {
    fn ext() -> &'static str;
}

impl <L: Lang> LangSyntaxExt<L> {
    fn create_template(name: &str, compiled_views: Vec<CompiledView>) -> LangSyntaxExt<L> {
        let ext_name = &format!("emit_{}_view_{}", L::ext(), name);
        let lang_ext = LangSyntaxExt {
            decl: NamedTemplateDecl { name: String::from(name), compiled_views: compiled_views },
            _l: PhantomData
        };

        lang_ext
    }
}

#[derive(Debug, Clone)]
struct LangSyntaxExt<L: Lang> {
    decl: NamedTemplateDecl,
    _l: PhantomData<L>
}

macro_rules! lang_expander {
    ($lang: ty) => (
        impl TTMacroExpander for LangSyntaxExt<$lang> {
            fn expand<'cx>(&self, ecx: &'cx mut ExtCtxt, span: Span, tts: &[TokenTree]) -> Box<MacResult + 'cx> {
               // self.decl.expand(ecx, span, tts)
                let mut parser = ecx.new_parser_from_tts(tts);
                let w_ident = parser.parse_ident().unwrap();
                self.create_write_block(ecx, w_ident)
            }
        }
    )
}
lang_expander!(Rust);
lang_expander!(Js);

macro_rules! lang {
    ($lang: ty, $ext: expr) => (
        impl Lang for $lang {
            fn ext() -> &'static str { $ext }
        }
    )
}
lang!(Rust, "rust");
lang!(Js, "js");

impl WriteBlockFactory for LangSyntaxExt<Rust> {
    fn create_write_block<'cx>(&self, ecx: &'cx mut ExtCtxt, w_ident: ast::Ident) -> Box<MacResult + 'cx> {
        codegen::create_template_write_block(ecx, w_ident, &self.decl.compiled_views)
    }
}

impl WriteBlockFactory for LangSyntaxExt<Js> {
    fn create_write_block<'cx>(&self, ecx: &'cx mut ExtCtxt, w_ident: ast::Ident) -> Box<MacResult + 'cx> {
        let funcs: Vec<String> = self.decl.compiled_views.iter()
            .map(|compiled_view| compiled_view.into_js_function(ecx))
            .intersperse("; ".into())
            .collect();

        codegen::create_write_statements_block(ecx, w_ident, funcs.as_slice())
    }
}

impl TTMacroExpander for NamedTemplateDecl {
    fn expand<'cx>(&self, ecx: &'cx mut ExtCtxt, span: Span, tts: &[TokenTree]) -> Box<MacResult + 'cx> {
        let mut parser = ecx.new_parser_from_tts(tts);
        let w_ident = parser.parse_ident().unwrap();
        //let w_ident = ecx.ident_of("out");
        codegen::create_template_write_block(ecx, w_ident, &self.compiled_views)
    }
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
    //let ident = ecx.ident_of(name.to_owned());
    let mut parser = ecx.new_parser_from_tts(&tts);
    //let tokens: Vec<TokenTree> = tts.into();
    match parse_template_into_compiled_view(ecx, span, &mut parser) {
        Ok(compiled_view) => {
            let compiled_views = vec![compiled_view];
            define_named_template!(ecx, ident, Rust, "rust", compiled_views);
            define_named_template!(ecx, ident, Js, "js", compiled_views);
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