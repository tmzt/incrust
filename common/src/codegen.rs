
use syntax::ast;
use syntax::codemap::{Span, DUMMY_SP};
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult};
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;

use compiled_view::CompiledView;


pub trait IntoViewItem {
    fn into_view_item<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Item>;
}

pub trait IntoWriteStmt {
    fn into_write_stmt<'cx>(&self, ecx: &'cx ExtCtxt, w: ast::Ident) -> ast::Stmt;
}

pub fn create_template_block<'cx>(ecx: &'cx ExtCtxt,
                              compiled_views: Vec<&CompiledView>)
                              -> Box<MacResult + 'cx> {
    let view_item_stmts: Vec<ast::Stmt> = compiled_views.iter()
        .map(|compiled_view| compiled_view.into_view_item(ecx))
        .map(|item| ecx.stmt_item(DUMMY_SP, item))
        .collect();

    let mut stmts = Vec::new();
    stmts.extend(view_item_stmts);

    let name = ecx.ident_of("rusttemplate_view_root");
    let args = vec![];
    let call_expr = ecx.expr_call_ident(DUMMY_SP, name, args);
    stmts.push(ecx.stmt_expr(call_expr));

    let block = ecx.block(DUMMY_SP, stmts);

    MacEager::expr(ecx.expr_block(block))
}
