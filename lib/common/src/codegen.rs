
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

pub mod lang {
    use std::marker::PhantomData;
    pub enum Html {}
    pub enum Js {}

    pub trait Lang {
        fn ext() -> &'static str;
    }

    macro_rules! lang {
        ($lang: ty, $ext: expr) => (
            impl Lang for $lang {
                fn ext() -> &'static str { $ext }
            }
        )
    }
    lang!(Html, "html");
    lang!(Js, "js");

    #[derive(Debug, Clone)]
    pub struct NamedOutputExt<L: Lang, D> {
        data: D,
        _l: PhantomData<L>
    }

    impl <L: Lang, D> NamedOutputExt<L, D> {
        pub fn create_named_output(name: &str, data: D) -> NamedOutputExt<L, D> {
            NamedOutputExt {
                data: data,
                _l: PhantomData::<L>,
            }
        }
    }
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
    use super::lang::Lang;

    /// Request the implementer to write itself out as strings
    pub trait WriteStrings<L: Lang> {
        fn write_strings<'s, 'cx>(&self, ecx: &'cx ExtCtxt<'cx>, w: &'s mut StringWrite<L>);
    }

    pub trait StringWrite<L: Lang> {
        fn write_string<'cx>(&mut self, ecx: &'cx ExtCtxt, contents: &str);
    }

    mod internal {
        use super::super::stmt_writer::{WriteStmts, StmtWrite};
        use super::super::lang::{Lang, Html, Js};
        use super::{WriteStrings, StringWrite};
        use syntax::ext::base::ExtCtxt;
        use syntax::ast;

        struct StmtsWrapper<'s> {
            writer: ast::Ident,
            inner: &'s mut StmtWrite
        }

        macro_rules! string_writer_impl (
            ($lang: ident) => (
                impl<'s> StringWrite<$lang> for StmtsWrapper<'s> {
                    fn write_string<'cx>(&mut self, ecx: &'cx ExtCtxt, contents: &str) {
                        let writer = &self.writer;

                        let stmt = quote_stmt!(ecx, {
                            write!($writer, "{}", $contents).unwrap();
                        }).unwrap();

                        self.inner.write_stmt(stmt);
                    }
                }

                impl<S: WriteStrings<$lang>> WriteStmts for S {
                    fn write_stmts<'s, 'cx>(&self, ecx: &'cx ExtCtxt<'cx>, w: &'s mut StmtWrite) {
                        let writer = ecx.ident_of("writer");
                        let mut wrapper = StmtsWrapper { writer: writer, inner: w };

                        self.write_strings(ecx, &mut wrapper);
                    }
                }
            )
        );
        string_writer_impl!(Html);

        /*
        impl<'s, L: Lang> StringWrite<L> for StmtsWrapper<'s> {
            fn write_string<'cx>(&mut self, ecx: &'cx ExtCtxt, contents: &str) {
                let writer = &self.writer;

                let stmt = quote_stmt!(ecx, {
                    write!($writer, "{}", $contents).unwrap();
                }).unwrap();

                self.inner.write_stmt(stmt);
            }
        }

        impl<L: Lang> WriteStmts for WriteStrings<L> {
            fn write_stmts<'s, 'cx>(&self, ecx: &'cx ExtCtxt<'cx>, w: &'s mut StmtWrite) {
                let writer = ecx.ident_of("writer");
                let mut wrapper = StmtsWrapper { writer: writer, inner: w };

                self.write_strings(ecx, &mut wrapper);
            }
        }
        */
    }
}

pub mod block_writer {
    use super::stmt_writer::{WriteStmts, StmtWrite};
    use syntax::codemap::{Span, DUMMY_SP};
    use syntax::ext::base::ExtCtxt;
    use syntax::ast;
    use syntax::ptr::P;
    use syntax::ext::build::AstBuilder;

    /// Request the implementer to write itself out as blocks
    pub trait WriteBlocks {
        fn write_blocks<'s, 'cx>(&self, ecx: &'cx ExtCtxt<'cx>, w: &'s mut BlockWrite);
    }

    pub trait BlockWrite {
        fn write_block<'cx>(&mut self, ecx: &'cx ExtCtxt, block: P<ast::Block>);
    }

    impl<'s> BlockWrite for Vec<P<ast::Block>> {
        fn write_block<'cx>(&mut self, ecx: &'cx ExtCtxt, block: P<ast::Block>) {
            self.push(block);
        }
    }

    impl<S: WriteStmts> WriteBlocks for S {
        fn write_blocks<'s, 'cx>(&self, ecx: &'cx ExtCtxt<'cx>, w: &'s mut BlockWrite) {
            let mut stmts = vec![];
            &self.write_stmts(ecx, &mut stmts);

            let block = ecx.block(DUMMY_SP, stmts);
            w.write_block(ecx, block);
        }
    }
}

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

pub mod named_output_writer {
    use super::stmt_writer::WriteStmts;
    use super::lang::{Lang, NamedOutputExt};
    use syntax::codemap::{Span, DUMMY_SP};
    use syntax::ext::base::ExtCtxt;
    use syntax::ext::build::AstBuilder;

    pub trait WriteNamedOutputs<L : Lang, D> {
        fn write_named_outputs<'cx>(&self, ecx: &'cx ExtCtxt<'cx>, w: &mut NamedOutputWrite<D>);
    }

    pub trait NamedOutputWrite<D> {
        fn write_named_output(&mut self, name: &str, data: D);
    }

    impl<L: Lang, D> NamedOutputWrite<D> for Vec<NamedOutputExt<L, D>> {
        fn write_named_output(&mut self, name: &str, data: D) {
            let ext = NamedOutputExt::<L, D>::create_named_output(name, data);
            self.push(ext);
        }
    }

    impl<S: WriteStmts, L: Lang, D> WriteNamedOutputs<L, D> for S {
        fn write_named_outputs<'cx>(&self, ecx: &'cx ExtCtxt<'cx>, w: &mut NamedOutputWrite<D>) {
            let mut stmts = vec![];
            &self.write_stmts(ecx, &mut stmts);
        }
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
