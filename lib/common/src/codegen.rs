
use syntax::ast;
use syntax::codemap::{Span, DUMMY_SP};
use syntax::util::small_vector::SmallVector;
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult};
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;

use nodes::view_node::View;
use nodes::template_node::Template;


pub trait IntoViewItem {
    fn into_view_item<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Item>;
}

pub trait IntoWriteStmt {
    fn into_write_stmt<'cx>(&self, ecx: &'cx ExtCtxt, w: ast::Ident) -> ast::Stmt;
}

pub trait IntoBlock {
    fn into_block<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Block>;
}

pub mod lang {
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
}

pub mod ext {
    use std::marker::PhantomData;
    use super::lang::Lang;
    use syntax::ext::base::ExtCtxt;
    use syntax::ast;

    #[derive(Debug, Clone)]
    pub struct NamedOutputExt<L: Lang, D> {
        name: String,
        data: D,
        _l: PhantomData<L>
    }

    impl <L: Lang, D> NamedOutputExt<L, D> {
        pub fn name(&self) -> &str {
            &self.name
        }

        pub fn create_named_output(name: &str, data: D) -> NamedOutputExt<L, D> {
            NamedOutputExt {
                name: name.to_owned(),
                data: data,
                _l: PhantomData::<L>,
            }
        }
    }
}

pub mod output_stmt_writer {
    use syntax::ext::base::ExtCtxt;
    use syntax::ast;
    use super::lang::{Lang, Html, Js};

    pub trait WriteOutputStmts<L: Lang> {
        fn write_output_stmts<'s, 'cx>(&self, ecx: &'cx ExtCtxt, w: &'s mut OutputStmtWrite<L>, writer: ast::Ident);
    }

    pub trait OutputStmtWrite<L: Lang> {
        fn write_output_stmt(&mut self, stmt: ast::Stmt);
    }

    macro_rules! stmt_writer_impl (
        ($lang: ident) => (
            impl<'s> OutputStmtWrite<$lang> for Vec<ast::Stmt> {
                fn write_output_stmt(&mut self, stmt: ast::Stmt) {
                    self.push(stmt);
                }
            }
        )
    );
    stmt_writer_impl!(Html);
}

pub mod output_string_writer {
    use std::iter::Iterator;
    use syntax::ext::base::ExtCtxt;
    use syntax::ast;
    use super::output_stmt_writer::{WriteOutputStmts, OutputStmtWrite};
    use super::lang::{Lang, Html, Js};

    use output_actions::{WriteOutputActions, OutputAction};

    pub trait WriteOutputStrings<L: Lang> {
        fn write_output_strings<'s, 'cx>(&self, ecx: &'cx ExtCtxt, w: &'s mut OutputStringWrite<L>);
    }

    pub trait OutputStringWrite<L: Lang> {
        fn write_output_string<'cx>(&mut self, ecx: &'cx ExtCtxt, contents: &str);
    }

    macro_rules! string_writer_impl (
        ($lang: ident) => (
            impl<S: WriteOutputStrings<$lang>> WriteOutputStmts<$lang> for S {
                fn write_output_stmts<'s, 'cx>(&self, ecx: &'cx ExtCtxt, w: &'s mut OutputStmtWrite<$lang>, writer: ast::Ident) {
                    let mut output_strings: Vec<String> = vec![];
                    &self.write_output_strings(ecx, &mut output_strings);

                    let stmts = output_strings.iter()
                        .map(|s| {
                            quote_stmt!(ecx, {
                                write!($writer, "{}", &s).unwrap();
                            }).unwrap()
                        });

                    for stmt in stmts {
                        w.write_output_stmt(stmt);
                    }
                }
            }

            impl OutputStringWrite<$lang> for Vec<String> {
                fn write_output_string<'cx>(&mut self, ecx: &'cx ExtCtxt, contents: &str) {
                    self.push(contents.to_owned());
                }
            }
        )
    );
    string_writer_impl!(Html);
}

pub mod block_writer {
    use syntax::ast;
    use syntax::ptr::P;
    use syntax::ext::base::ExtCtxt;

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

    /*
    impl IntoBlock for View {
        fn into_block<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Block> {
            let mut stmts = vec![];
            let w_ident = ecx.ident_of("out");
            &self.write_stmts(ecx, &mut stmts);
            stmts.push(quote_stmt!(ecx, let mut $w_ident = String::new()).unwrap());
            ecx.block(DUMMY_SP, stmts)            
        }
    }
    */
}

pub mod item_writer {
    use super::block_writer::WriteBlocks;
    use syntax::codemap::{Span, DUMMY_SP};
    use syntax::ext::base::ExtCtxt;
    use syntax::ast;
    use syntax::ptr::P;
    use syntax::ext::build::AstBuilder;
    use codegen::IntoBlock;
    use nodes::view_node::View;
    use nodes::template_node::Template;

    // Request the implement to write itself out as items
    pub trait WriteItems {
        fn write_items<'cx>(&self, ecx: &'cx ExtCtxt, w: &mut ItemWrite);
    }

    pub trait ItemWrite {
        fn write_item<'cx>(&mut self, ecx: &'cx ExtCtxt, item: P<ast::Item>);
    }

    impl<'s> ItemWrite for Vec<P<ast::Item>> {
        fn write_item<'cx>(&mut self, ecx: &'cx ExtCtxt, item: P<ast::Item>) {
            self.push(item);
        }
    }

    fn create_view_item<'cx>(ecx: &'cx ExtCtxt<'cx>, span: Span, view: &View) -> P<ast::Item> {
        let name = ecx.ident_of(&format!("rusttemplate_view_{}", view.name()));
        let block = view.into_block(ecx);

        let inputs = vec![];
        let ret_ty = quote_ty!(ecx, String);
        ecx.item_fn(span, name, inputs, ret_ty, block)
    }

    fn write_item<'s, 'cx>(view: &View, ecx: &'cx mut ExtCtxt<'cx>, w: &'s mut ItemWrite) {
        let item = create_view_item(ecx, DUMMY_SP, view);
        w.write_item(ecx, item);
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
    use super::lang::{Lang, Html, Js};
    use super::output_string_writer::{WriteOutputStrings, OutputStringWrite};
    use super::output_stmt_writer::{WriteOutputStmts, OutputStmtWrite};
    use super::ext::NamedOutputExt;
    use syntax::codemap::{Span, DUMMY_SP};
    use syntax::ext::base::ExtCtxt;
    use syntax::ext::build::AstBuilder;

    pub trait WriteNamedOutputs<L : Lang, D> {
        fn write_named_outputs<'cx>(&self, ecx: &'cx ExtCtxt<'cx>, w: &mut NamedOutputWrite<D>);
    }

    pub trait NamedOutputWrite<D> {
        fn write_named_output(&mut self, name: &str, data: D);
    }

    macro_rules! lang_impl (
        ($lang: ident, $item: ty) => (
            impl NamedOutputWrite<$item> for Vec<NamedOutputExt<$lang, $item>> {
                fn write_named_output(&mut self, name: &str, data: $item) {
                    let ext = NamedOutputExt::<$lang, $item>::create_named_output(name, data);
                    self.push(ext);
                }
            }

            impl<S: WriteOutputStmts<$lang>> WriteNamedOutputs<$lang, $item> for S {
                fn write_named_outputs<'cx>(&self, ecx: &'cx ExtCtxt, w: &mut NamedOutputWrite<$item>) {
                    let mut stmts = vec![];
                    let writer = ecx.ident_of("out");
                    &self.write_output_stmts(ecx, &mut stmts, writer);
                }
            }
        )
    );
    lang_impl!(Html, String);
}

pub fn create_template_result<'cx>(ecx: &'cx mut ExtCtxt<'cx>, w_ident: ast::Ident, template: &Template) -> Box<MacResult + 'cx> {
    use self::item_writer::WriteItems;

    let mut items: Vec<P<ast::Item>> = vec![];
    template.write_items(ecx, &mut items);
    MacEager::items(SmallVector::many(items))
}
