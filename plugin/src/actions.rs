use rustc_plugin::Registry;

use syntax::ext::base::IdentTT;
use syntax::parse::token;

use syntax::ast;
use syntax::ast::ExprKind;
use syntax::ptr::P;
use syntax::codemap::{Span, DUMMY_SP};
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult, SyntaxExtension, NormalTT, TTMacroExpander};
use syntax::ext::quote::rt::ToTokens;
use syntax::ext::build::AstBuilder;
use syntax::ext::hygiene::Mark;
use syntax::tokenstream::TokenTree;
use syntax::util::ThinVec;
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

        let expr_tokens = $e.to_tokens($ecx);
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

fn parse_default_expr<'cx, 'a>(ecx: &'cx mut ExtCtxt, span: Span, parser: &mut Parser<'a>) -> PResult<'a, P<ast::Expr>> {
    // Expression
    parser.parse_expr()
}

fn process_raw_expr<'cx, 'a>(ecx: &'cx mut ExtCtxt<'a>, span: Span, raw_expr: Vec<TokenTree>, store_name: &str) -> Vec<TokenTree> {
    let mut parser = ecx.new_parser_from_tts(&raw_expr);
    let mut res: Vec<TokenTree> = vec![];

    match parser.parse_expr() {
        Ok(p) => {
            match p.node {
                ast::ExprKind::Struct(ref pth, _, _) => {
                    let pth_name = format!("{}", pth).to_lowercase();
                    if pth_name != store_name {
                        ecx.span_err(span, &format!("Struct name must match store name (ignoring case), got '{}' expected '{}'", pth_name, store_name));
                    }
                    let expr_tokens: Vec<TokenTree> = p.to_tokens(ecx);
                    ecx.span_warn(span, &format!("Expr Tokens: {:?}", expr_tokens));

                    let expr_tts = expr_tokens[0].get_tt(0);
                    ecx.span_warn(span, &format!("Expr TTS: {:?}", expr_tts));
                    //res.extend(expr_tokens.iter().skip(1).cloned());
                },
                _ => {}
            };
        },
        _ => {}
    };
    res
    /*
    match parser.token {
        ast::ExprKind::Struct(pth, _, _) => {
            let pth_name = format!("{}", pth).to_lowercase();
            if pth_name != store_name {
                ecx.span_err(span, &format!("Struct name must match store name (ignoring case), got '{}' expected '{}'", pth_name, store_name));
                return vec![];
            }
            raw_expr[1..].into()
        },
        _ => raw_expr
    }
    */
}

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
    let raw_expr = try!(parse_limited_expr_block(ecx, span, &store_name, parser));
    let expr = process_raw_expr(ecx, span, raw_expr, &store_name);

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
