use rustc_plugin::Registry;

use syntax::ext::base::IdentTT;
use syntax::parse::token;

use syntax::ast;
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
