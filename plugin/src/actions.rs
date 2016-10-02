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
use syntax::parse::common::SeqSep;

use expr::parse_limited_expr_block;

use itertools::Itertools;


pub fn register_store(reg: &mut Registry) {
    // Store definitions
    reg.register_syntax_extension(token::intern("define_store_action"),
            NormalTT(Box::new(expand_define_store_action), None, false));
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
        let out = $w;

        //let expr_tokens = action_expr_to_tts($ecx, $e);
        let expr_tokens = $e;
        quote_stmt!($ecx, {
            let expr = stringify!($expr_tokens);
            write!($out, "\t\tcase '{}': return {};\n",
                $q_action_name,
                format!("{{ {}: ({}) }}",
                    $q_store_name,
                    expr
                )
            ).unwrap();
        })
    })
}

/*
fn parse_action_expr<'cx, 'a>(ecx: &'cx mut ExtCtxt, span: Span, parser: &mut Parser<'a>) -> PResult<'a, P<ast::Expr>> {
    // Expression
    parser.parse_expr()
}
*/

fn parse_default_expr<'cx, 'a>(ecx: &'cx mut ExtCtxt, span: Span, parser: &mut Parser<'a>) -> PResult<'a, P<ast::Expr>> {
    // Expression
    parser.parse_expr()
}

/*
fn parse_limited_expr_block<'cx, 'a>(ecx: &'cx mut ExtCtxt<'a>, span: Span, store_name: &str, parser: &mut Parser<'a>) -> PResult<'a, Vec<TokenTree>> {
    try!(parser.expect(&token::OpenDelim(token::Brace)));
    let expr_tokens = parser.parse_seq_to_end(&token::CloseDelim(token::Brace),
                          SeqSep::none(),
                          |parser| parser.parse_token_tree()).unwrap();
    parser.eat(&token::CloseDelim(token::Brace));
    let mut expr_parser = ecx.new_parser_from_tts(&expr_tokens);
    Ok(try!(parse_limited_expr(ecx, store_name, span, &mut expr_parser)))
}
*/

fn parse_define_store_action<'cx, 'a>(ecx: &'cx mut ExtCtxt<'a>, span: Span, parser: &mut Parser<'a>) -> PResult<'a, Box<MacResult + 'cx>> {
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
    //let expr = try!(parse_action_expr_block(ecx, span, &store_name, parser));
    let expr = try!(parse_limited_expr_block(ecx, span, &store_name, parser));

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
