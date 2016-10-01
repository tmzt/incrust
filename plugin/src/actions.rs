use rustc_plugin::Registry;

use syntax::ext::base::IdentTT;
use syntax::parse::token;

use syntax::ast;
use syntax::ptr::P;
use syntax::codemap::{Span, DUMMY_SP};
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult, SyntaxExtension, NormalTT, TTMacroExpander};
use syntax::ext::build::AstBuilder;
use syntax::ext::hygiene::Mark;
use syntax::tokenstream::TokenTree;
use syntax::util::small_vector::SmallVector;
use syntax::parse::PResult;
use syntax::parse::parser::Parser;

use itertools::Itertools;


pub fn register_store(reg: &mut Registry) {
    // Store definitions
    reg.register_syntax_extension(token::intern("define_store_action"),
            NormalTT(Box::new(expand_define_store_action), None, false));

//    reg.register_syntax_extension(token::intern("define_store"),
//            IdentTT(Box::new(expand_define_store), None, false));
}

trait Store {
}

trait StoreAction {
    type Store: Store;
    
    fn name() -> &'static str;
    fn apply_mut(store: &mut Self::Store);
}

macro_rules! write_action_case {
    ($ecx: expr, $w: expr, $store_name:expr, $action_name:expr, $e:expr) => ({
        let q_store_name = $store_name;
        let q_action_name = $action_name;
        let q_expr = $e;
        let out = $w;
        quote_stmt!($ecx, {
            //write!($w, "test");
            //write!($w, "\t\tcase '{}': return {};\n", "1", "2");
            write!($out, "\t\tcase '{}': return {};\n",
                $q_action_name,
                format!("{{ {}: {} }}",
                    $q_store_name,
                    stringify!($q_expr)
                )
            )
        })
    })
}

/// TODO: Document
pub fn expand_define_store<'cx>(ecx: &'cx mut ExtCtxt, span: Span, ident: ast::Ident, tts: Vec<TokenTree>) -> Box<MacResult + 'cx> {
    let mut parser = ecx.new_parser_from_tts(&tts);

    let store_name = ident.name.to_string();
    let struct_name = format!("{}Store", store_name);
    let struct_ident = ecx.ident_of(&struct_name[..]);

    let stmt = quote_stmt!(ecx, {
        struct $struct_ident {
            
        }
    }).unwrap();
    let stmts = vec![stmt];
    let block = ecx.block(DUMMY_SP, stmts);
    MacEager::expr(ecx.expr_block(block))
}

fn parse_action_expr<'cx, 'a>(ecx: &'cx mut ExtCtxt, span: Span, parser: &mut Parser<'a>) -> PResult<'a, P<ast::Expr>> {
    // Expression
    parser.parse_expr()
}

fn parse_default_expr<'cx, 'a>(ecx: &'cx mut ExtCtxt, span: Span, parser: &mut Parser<'a>) -> PResult<'a, P<ast::Expr>> {
    // Expression
    parser.parse_expr()
}

fn parse_define_store_action<'cx, 'a>(ecx: &'cx mut ExtCtxt, span: Span, parser: &mut Parser<'a>) -> PResult<'a, Box<MacResult + 'cx>> {
    // Output stream
    let w = try!(parser.parse_ident());
    try!(parser.expect(&token::Comma));

    // Store identifier
    let store_name = try!(parser.parse_ident()).name.to_string();
    try!(parser.expect(&token::Comma));

    // Token 'action'
    try!(parser.parse_ident());

    // Action name
    let action_name = try!(parser.parse_ident()).name.to_string();

    // =>
    try!(parser.expect(&token::FatArrow));

    // expression
    let expr = try!(parse_action_expr(ecx, span, parser));

    let stmt = write_action_case!(ecx, w, store_name, action_name, expr).unwrap();

    let stmts = vec![stmt];
    let block = ecx.block(DUMMY_SP, stmts);
    Ok(MacEager::expr(ecx.expr_block(block)))
}

pub fn expand_define_store_action<'cx>(ecx: &'cx mut ExtCtxt, span: Span, tts: &[TokenTree]) -> Box<MacResult + 'cx> {
    let mut parser = ecx.new_parser_from_tts(&tts);
    match parse_define_store_action(ecx, span, &mut parser) {
        Ok(mac_result) => mac_result,
        Err(mut err) => {
            err.emit();
            DummyResult::any(span)
        }
    }
}
