
use syntax::ast;
use syntax::codemap::{Span, DUMMY_SP};
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult};
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;

use view::View;


pub trait IntoViewItem {
    fn into_view_item<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Item>;
}

pub trait IntoWriteStmt {
    fn into_write_stmt<'cx>(&self, ecx: &'cx ExtCtxt, w: ast::Ident) -> ast::Stmt;
}

pub fn create_template_block<'cx>(ecx: &'cx ExtCtxt,
                              span: Span,
                              views: Vec<View>)
                              -> Box<MacResult + 'cx> {
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

    let block = ecx.block(span, stmts);

    MacEager::expr(ecx.expr_block(block))
}
