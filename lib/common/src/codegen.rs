
use syntax::ast;
use syntax::codemap::{Span, DUMMY_SP};
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult};
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;

use nodes::view_node::View;


pub trait IntoViewItem {
    fn into_view_item<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Item>;
}

pub trait IntoWriteStmt {
    fn into_write_stmt<'cx>(&self, ecx: &'cx ExtCtxt, w: ast::Ident) -> ast::Stmt;
}

pub trait IntoBlock {
    fn into_block<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Block>;
}

pub trait WriteItems {
    fn write_items<'cx>(&self, ecx: &'cx mut ExtCtxt);
}

pub mod stmt_writer {
    use syntax::ext::base::ExtCtxt;
    use syntax::ast;

    pub trait WriteStmts {
        fn write_stmts<'s, 'cx>(&self, ecx: &'cx ExtCtxt<'cx>, w: &'s mut StmtWrite);
    }

    pub trait StmtWrite {
        fn write_stmt(&mut self, stmt: ast::Stmt);
    }

    impl<'s> StmtWrite for Vec<ast::Stmt> {
        fn write_stmt(&mut self, stmt: ast::Stmt) {
            self.push(stmt);
        }
    }
}

pub mod string_writer {
    use syntax::ext::base::ExtCtxt;
    use syntax::ast;

    /// Request the implementer to write itself out as strings
    pub trait WriteStrings {
        fn write_strings<'s, 'cx>(&self, ecx: &'cx ExtCtxt<'cx>, w: &'s mut StringWrite);
    }

    pub trait StringWrite {
        fn write_string<'cx>(&mut self, ecx: &'cx ExtCtxt, contents: &str);
    }

    /*
    /// Request the implementer to write itself out as strings, by providing a closure
    /// which will be provided with a writer.
    ///
    /// This is required to capture the needed $writer expression in the resulting statement.
    pub trait WriteStringsUsing {
        fn write_strings_using<'s, 'cx>(&self, ecx: &'cx ExtCtxt<'cx>, w: &'s mut StringWrite);
    }

    pub trait WriteStringUsing {
        fn write_string_using<'cx>(&mut self, ecx: &'cx ExtCtxt, f: Fn(&mut StringWriter));
    }
    */

    mod internal {
        use super::super::stmt_writer::{WriteStmts, StmtWrite};
        use super::{WriteStrings, StringWrite};
        use syntax::ext::base::ExtCtxt;
        use syntax::ast;

        struct StmtsWrapper<'s> {
            writer: ast::Ident,
            inner: &'s mut StmtWrite
        }

        impl<'s> StringWrite for StmtsWrapper<'s> {
            fn write_string<'cx>(&mut self, ecx: &'cx ExtCtxt, contents: &str) {
                let writer = &self.writer;

                let stmt = quote_stmt!(ecx, {
                    write!($writer, "{}", $contents).unwrap();
                }).unwrap();

                self.inner.write_stmt(stmt);
            }
        }

        impl<S: WriteStrings> WriteStmts for S {
            fn write_stmts<'s, 'cx>(&self, ecx: &'cx ExtCtxt<'cx>, w: &'s mut StmtWrite) {
                let writer = ecx.ident_of("writer");
                let mut wrapper = StmtsWrapper { writer: writer, inner: w };

                self.write_strings(ecx, &mut wrapper);
            }
        }
    }

    /*
    mod emitter {
        use super::{WriteStrings, StringWrite};
        use syntax::ext::base::ExtCtxt;
        use syntax::ast;

        struct Wrapper<'cx> {
            ecx: &'cx ExtCtxt<'cx>,
            writer: ast::Ident
        }

        trait EmitWrites {
            fn emit_writes<'cx>(&self, ecx: &'cx ExtCtxt<'cx>);
        }

        impl<'cx> StringWrite for Wrapper<'cx> {
            fn write_string<'cx>(&mut self, ecx: &'cx ExtCtxt, contents: &str) {
                let writer = &self.writer;

                let stmt = quote_stmt!(ecx, {
                    write!($writer, "{}", $contents).unwrap();
                }).unwrap();

                // TODO: Emit
            }
        }

        impl<S: WriteStrings> EmitWrites for S {
            fn emit_writes<'cx>(&self, ecx: &'cx ExtCtxt<'cx>) {
                let writer = ecx.ident_of("writer");
                let wrapper = Wrapper { ecx: ecx, writer: writer };

                self.write_strings(ecx, &mut wrapper);
            }
        }
    }
    */
}

/*
mod string_output {
    use super::StringOutputStmtWrite;
    use syntax::ext::base::ExtCtxt;
    use syntax::ast;

    impl StringOutputStmtWrite<String> for Vec<ast::Stmt> {
        fn write_string_output_stmt<'cx>(&mut self, ecx: &'cx mut ExtCtxt, writer: ast::Ident, contents: String) {
            let stmt = quote_stmt!(ecx, {
                write!($writer, "{}", $contents).unwrap();
            }).unwrap();

            self.push(stmt);
        }
    }
}
*/

mod utils {
    use std::rc::Rc;
    use syntax::ext::base::ExtCtxt;
    use syntax::ext::build::AstBuilder;
    use syntax::ext::base::{SyntaxExtension};

    pub fn define_ext<'cx>(ecx: &'cx mut ExtCtxt, name: &str, ext: Rc<SyntaxExtension>) {
        let ident = ecx.ident_of(name);
        (*ecx.resolver).add_ext(ident, ext);
    }
}

/*
fn create_view_item_stmts<'cx>(ecx: &'cx ExtCtxt, views: &[View]) -> Vec<ast::Stmt> {
    let view_item_stmts: Vec<ast::Stmt> = views.iter()
        .map(|view| view.into_view_item(ecx))
        .map(|item| ecx.stmt_item(DUMMY_SP, item))
        .collect();

    view_item_stmts
}

pub fn create_template_block<'cx>(ecx: &'cx ExtCtxt,
                              views: &[View],
                              call_root: bool)
                              -> Box<MacResult + 'cx> {
    let view_item_stmts = create_view_item_stmts(ecx, views);

    let mut stmts = Vec::new();
    stmts.extend(view_item_stmts);

    if call_root {
        let name = ecx.ident_of("rusttemplate_view_root");
        let args = vec![];
        let call_expr = ecx.expr_call_ident(DUMMY_SP, name, args);
        stmts.push(ecx.stmt_expr(call_expr));
    }

    let block = ecx.block(DUMMY_SP, stmts);

    MacEager::expr(ecx.expr_block(block))
}

pub fn create_template_write_block<'cx>(ecx: &'cx ExtCtxt,
                              w_ident: ast::Ident,
                              views: &[View])
                              -> Box<MacResult + 'cx> {
    let view_item_stmts = create_view_item_stmts(ecx, views);

    let mut stmts = Vec::new();
    stmts.extend(view_item_stmts);

    let name = ecx.ident_of("rusttemplate_view_root");
    let args = vec![];
    let call_expr = ecx.expr_call_ident(DUMMY_SP, name, args);
    let write_stmt = quote_stmt!(ecx, {
        write!($w_ident, "{}", $call_expr).unwrap();
    }).unwrap();

    stmts.push(write_stmt);
    let block = ecx.block(DUMMY_SP, stmts);

    MacEager::expr(ecx.expr_block(block))
}

pub fn create_write_statements_block<'cx>(ecx: &'cx ExtCtxt, w_ident: ast::Ident, s: &[String]) -> Box<MacResult + 'cx> {
    let write_stmts = s.iter()
        .map(|s| {
            quote_stmt!(ecx, {
                write!($w_ident, "{}", $s).unwrap();
            }).unwrap()
        }).collect();

    let block = ecx.block(DUMMY_SP, write_stmts);
    MacEager::expr(ecx.expr_block(block))
}
*/
