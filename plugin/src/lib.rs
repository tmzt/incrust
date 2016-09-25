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
use syntax::parse::{token, PResult};
use syntax::print::pprust::{token_to_string, tts_to_string};
use syntax::tokenstream::TokenTree;
use syntax::util::small_vector::SmallVector;
use syntax::ptr::P;


#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    //reg.register_macro("jsr_call", jsr_call);
    reg.register_macro("emit_rust_template", emit_rust_template);
}

fn rust_template_fn_item<'a>(ecx: &'a ExtCtxt, span: Span) -> P<ast::Item> {
    let name = ecx.ident_of(&format!("rusttemplate_{}", "root"));

    let stmt = quote_stmt!(ecx,
        {
            println!("Testing");
        }
    ).unwrap();

    let args = "";
    let stmts = vec![ stmt ];

    let block = ecx.block(span, stmts);
    let inputs = vec![];
    //let ret_ty = ast::FunctionRetTy::Default(DUMMY_SP);
    let ret_ty = quote_ty!(ecx, ());

    ecx.item_fn(DUMMY_SP, name, inputs, ret_ty, block)
}

fn emit_rust_template<'cx>(
        ecx: &'cx mut ExtCtxt,
        span: Span,
        tts: &[TokenTree]) -> Box<MacResult + 'cx> {
    let mut i = 0;
    loop {
        match (tts.get(i), tts.get(i+1), tts.get(i+2)) {
            (Some(&TokenTree::Token(_, token::Ident(element_type))), _, _) => {
                ecx.span_warn(span, &format!("Outputing elementOpen for {}", &element_type.to_string()));
                let item = rust_template_fn_item(&ecx, span);

                let name = ecx.ident_of(&format!("rusttemplate_{}", "root"));
                let args = vec![];
                let call_expr = ecx.expr_call_ident(span, name, args);

                let block = ecx.block(span, vec![
                        ecx.stmt_item(span, item),
                        ecx.stmt_expr(call_expr)]);

                return MacEager::expr(ecx.expr_block(block));
            },
            (Some(_), _, _) => break,
            (None, _, _) => break
        }
    }

    //MacEager::stmts(SmallVector::many(result))
    //MacEager::items(SmallVector::many(items))
    DummyResult::any(span)
}


fn parse_js_call<F>(tts: &[TokenTree], f: &mut F) -> Vec<Spanned<String>>
        where F: FnMut(&str, &str, Span, &[TokenTree]) -> Vec<Spanned<String>> {
    let mut result = Vec::new();

    let mut i = 0;
    loop {
        match (tts.get(i), tts.get(i+1), tts.get(i+2)) {
            (Some(&TokenTree::Token(_, token::Ident(func_name))),
                Some(&TokenTree::Token(_, token::Ident(element_type))),
                Some(&TokenTree::Delimited(span, ref contents))) => {
                    i += 1;
                    result.extend(f(&func_name.to_string(), &element_type.to_string(), span, &contents.tts))
                },

/*
            (Some(&TokenTree::Token(span, ref tok)), _, _) => {
                            result.push(respan(span, token_to_string(tok)));
                        },
*/

            (Some(&TokenTree::Delimited(_, _)), _, _) => unimplemented!(),
            (Some(&TokenTree::Sequence(..)), _, _) => unimplemented!(),
            (Some(_), _, _) => break,
            (None, _, _) => break
        }
    }

    result
}

#[cfg(with_plugin)]
fn jsr_call<'cx>(
    ecx: &'cx mut ExtCtxt,
    span: Span,
    token_tree: &[TokenTree]) -> Box<MacResult + 'cx> {
        let mut items = Vec::new();

        parse_js_call(token_tree, &mut |func_name, element_type, span, tts| {
            println!("Call function {0} with element_type {1} \n", func_name, element_type);

            let s = token::intern("calling");
            //let s_token = ecx.expr(span, s);
            let s_lit = ecx.expr_lit(span, s);
            let s_token = ast::LitKind::Str(s, StrStyle::Cooked);

            let item = ecx.item(span,
                token::keywords::Invalid.ident(),
                Vec::new(),
                s_token);

            /*
            items.push(item);
            */

            /*
            let call_expr = match function.call_expr(ecx) {
                Ok(expr) => expr,
                Err(mut err) => {
                    err.emit();
                    return DummyResult::expr(span);
                }
            };

            let block = ecx.block(span,
                                vec![ecx.stmt_item(span, item),
                                    ecx.stmt_expr(call_expr)]);

            MacEager::expr(ecx.expr_block(block))
            */

            vec![s_token]
        });

        MacEager::items(SmallVector::many(items))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
