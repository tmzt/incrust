
use syntax::ast;
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult};

pub trait IntoWriteStmt {
    fn into_write_stmt<'cx>(&self, ecx: &'cx ExtCtxt, w: ast::Ident) -> ast::Stmt;
}
