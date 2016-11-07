
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

pub mod output_string_writer {
    use std::iter::Iterator;
    use syntax::codemap::{DUMMY_SP, Span};
    use syntax::ext::base::ExtCtxt;
    use syntax::ast;
    use super::lang::{Lang, Html, Js};

    pub trait WriteOutputStrings<L: Lang> {
        fn write_output_strings<'s, 'cx>(&self, ecx: &'cx ExtCtxt, w: &'s mut OutputStringWrite<L>);
    }

    pub trait OutputStringWrite<L: Lang> {
        fn write_output_string<'cx>(&mut self, ecx: &'cx ExtCtxt, contents: &str);
    }

    impl<L: Lang> OutputStringWrite<L> for Vec<String> {
        fn write_output_string<'cx>(&mut self, ecx: &'cx ExtCtxt, contents: &str) {
            self.push(contents.to_owned());
        }
    }

    impl<L: Lang> OutputStringWrite<L> for String {
        fn write_output_string<'cx>(&mut self, ecx: &'cx ExtCtxt, contents: &str) {
            self.push_str(&contents);
        }
    }
}

pub mod output_stmt_writer {
    use syntax::codemap::{Span, DUMMY_SP};
    use syntax::ext::base::ExtCtxt;
    use syntax::ast;
    use super::lang::{Lang, Html, Js};
    use super::output_string_writer::WriteOutputStrings;

    pub trait WriteOutputStmts<L: Lang> {
        fn write_output_stmts<'s, 'cx>(&self, ecx: &'cx ExtCtxt, w: &'s mut OutputStmtWrite<L>, writer: ast::Ident);
    }

    pub trait OutputStmtWrite<L: Lang> {
        fn write_output_stmt(&mut self, stmt: ast::Stmt);
    }

    impl<L: Lang, S: WriteOutputStrings<L>> WriteOutputStmts<L> for S {
        fn write_output_stmts<'s, 'cx>(&self, ecx: &'cx ExtCtxt, w: &'s mut OutputStmtWrite<L>, writer: ast::Ident) {
            let mut output_strings: Vec<String> = vec![];

            ecx.span_warn(DUMMY_SP, &format!("Writing output strings"));
            &self.write_output_strings(ecx, &mut output_strings);

            for output_string in &output_strings {
                ecx.span_warn(DUMMY_SP, &format!("Writing output string: {}", &output_string));
            }

            let stmts = output_strings.iter()
                .map(|s| {
                    quote_stmt!(ecx, {
                        write!($writer, "{}", &$s).unwrap();
                    }).unwrap()
                });

            for stmt in stmts {
                w.write_output_stmt(stmt);
            }
        }
    }

    impl<'s, L: Lang> OutputStmtWrite<L> for Vec<ast::Stmt> {
        fn write_output_stmt(&mut self, stmt: ast::Stmt) {
            self.push(stmt);
        }
    }

    /*
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
    */
}

pub mod output_block_writer {
    use syntax::codemap::{Span, DUMMY_SP};
    use syntax::ext::base::ExtCtxt;
    use syntax::ext::build::AstBuilder;
    use syntax::ast;
    use syntax::ptr::P;
    use super::lang::{Lang, Html, Js};
    use super::output_stmt_writer::WriteOutputStmts;

    pub trait IntoOutputBlock<L: Lang> {
        fn into_output_block<'cx>(&self, ecx: &'cx ExtCtxt, writer: ast::Ident) -> P<ast::Block>;
    }

    impl<L: Lang, S: WriteOutputStmts<L>> IntoOutputBlock<L> for S {
        fn into_output_block<'cx>(&self, ecx: &'cx ExtCtxt, writer: ast::Ident) -> P<ast::Block> {
            let mut out = Vec::new();
            out.push(quote_stmt!(ecx, let mut $writer = String::new()).unwrap());
            self.write_output_stmts(ecx, &mut out, writer);

            let expr = quote_expr!(ecx, $writer).unwrap();
            out.push(ecx.stmt_expr(P(expr)));
            ecx.block(DUMMY_SP, out)
        }
    }
}

pub mod output_item_writer {
    use super::output_stmt_writer::WriteOutputStmts;
    use super::output_block_writer::IntoOutputBlock;
    use syntax::codemap::{Span, DUMMY_SP};
    use syntax::ext::base::ExtCtxt;
    use syntax::ast;
    use syntax::ptr::P;
    use syntax::ext::build::AstBuilder;
    use codegen::IntoBlock;
    use codegen::lang::{Lang, Html, Js};
    use nodes::view_node::View;
    use nodes::template_node::Template;

    // Request the implement to write itself out as items
    pub trait WriteOutputItems<L: Lang> {
        fn write_output_items<'cx>(&self, ecx: &'cx ExtCtxt, w: &mut OutputItemWrite<L>);
    }

    pub trait OutputItemWrite<L: Lang> {
        fn write_output_item<'cx>(&mut self, ecx: &'cx ExtCtxt, item: P<ast::Item>);
    }

    pub trait IntoOutputItem<L: Lang> {
        fn into_output_item<'cx>(&self, ecx: &'cx ExtCtxt, name: &str) -> P<ast::Item>;
    }

    macro_rules! lang_impl (
        ($lang: ty) => (
            impl<S: WriteOutputStmts<$lang>> IntoOutputItem<$lang> for S {
                fn into_output_item<'cx>(&self, ecx: &'cx ExtCtxt, name: &str) -> P<ast::Item> {
                    let lang = stringify!($lang).to_lowercase();
                    let item_name = ecx.ident_of(&format!("rusttemplate_render_template_{}_view_{}_{}", "main", "root", &lang));
                    ecx.span_warn(DUMMY_SP, &format!("Writing item {}", item_name.to_string()));

                    let html_writer = ecx.ident_of("html_writer");
                    let js_writer = ecx.ident_of("js_writer");

                    let block = {
                        let mut out = Vec::new();

                        match lang {
                            _ if lang == "html" => {
                                self.write_output_stmts(ecx, &mut out, html_writer);
                            },
                            _ if lang == "js" => {
                                self.write_output_stmts(ecx, &mut out, js_writer);
                            },
                            _ => {
                                ecx.span_warn(DUMMY_SP, &format!("Unsupported language, won't render: {:?}", stringify!($lang)));
                            }
                        }
                        ecx.block(DUMMY_SP, out)
                    };

                    let item = {
                        let args = vec![
                            ecx.arg(DUMMY_SP, html_writer, quote_ty!(ecx, &mut String)),
                            ecx.arg(DUMMY_SP, js_writer, quote_ty!(ecx, &mut String)),
                        ];
                        let ret_ty = quote_ty!(ecx, ());
                        ecx.item_fn(DUMMY_SP, item_name, args, ret_ty, block)
                    };

                    item
                }
            }

            /*
            impl<S: IntoOutputBlock<$lang>> IntoOutputItem<$lang> for S {
                fn into_output_item<'cx>(&self, ecx: &'cx ExtCtxt, name: &str) -> P<ast::Item> {
                    //let item_name = ecx.ident_of(&format!("rusttemplate_{}_output_{}", stringify!($lang), name));
                    let lang = stringify!($lang).to_lowercase();
                    let item_name = ecx.ident_of(&format!("rusttemplate_render_template_{}_view_{}_{}", "main", "root", &lang));
                    ecx.span_warn(DUMMY_SP, &format!("Writing item {}", item_name.to_string()));

                    //let writer = ecx.ident_of("out");
                    //let block = self.into_output_block(ecx, writer);


                    let html_writer = ecx.ident_of("html_writer");
                    let js_writer = ecx.ident_of("js_writer");

                    let block = {
                        let mut out = Vec::new();

                        // TODO: Support JS and HTML output
                        self.write_output_stmts(ecx, &mut out, html_writer);
                        ecx.block(DUMMY_SP, out)
                    };

                    /*
                    let inputs = vec![];
                    let ret_ty = quote_ty!(ecx, String);
                    ecx.item_fn(DUMMY_SP, item_name, inputs, ret_ty, block)
                    */

                    let item = {
                        let args = vec![
                            ecx.arg(DUMMY_SP, html_writer, quote_ty!(ecx, &mut String)),
                            ecx.arg(DUMMY_SP, js_writer, quote_ty!(ecx, &mut String)),
                        ];
                        let ret_ty = quote_ty!(ecx, ());
                        ecx.item_fn(DUMMY_SP, item_name, args, ret_ty, block)
                    };

                    item
                }
            }
            */
        )
    );
    lang_impl!(Html);
    lang_impl!(Js);

    impl<'s, L: Lang> OutputItemWrite<L> for Vec<P<ast::Item>> {
        fn write_output_item<'cx>(&mut self, ecx: &'cx ExtCtxt, item: P<ast::Item>) {
            ecx.span_warn(DUMMY_SP, &format!("Writing output item: {:?}", &item));
            self.push(item);
        }
    }

    impl<L: Lang, S: IntoOutputItem<L>> WriteOutputItems<L> for S {
        fn write_output_items<'cx>(&self, ecx: &'cx ExtCtxt, w: &mut OutputItemWrite<L>) {
            let item = self.into_output_item(ecx, &"root");
            w.write_output_item(ecx, item);
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
    use super::lang::{Lang, Html, Js};
    use super::output_string_writer::{WriteOutputStrings, OutputStringWrite};
    use super::output_stmt_writer::{WriteOutputStmts, OutputStmtWrite};
    use super::ext::NamedOutputExt;
    use syntax::ast;
    use syntax::ptr::P;
    use syntax::ext::base::ExtCtxt;
    use syntax::ext::build::AstBuilder;
    use syntax::codemap::{Span, DUMMY_SP};

    pub trait WriteNamedOutputs<L : Lang, D> {
        fn write_named_outputs<'cx>(&self, ecx: &'cx ExtCtxt<'cx>, w: &mut NamedOutputWrite<D>, writer: ast::Ident);
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
                fn write_named_outputs<'cx>(&self, ecx: &'cx ExtCtxt, w: &mut NamedOutputWrite<$item>, writer: ast::Ident) {
                    let mut stmts = vec![];
                    &self.write_output_stmts(ecx, &mut stmts, writer);
                }
            }
        )
    );
    lang_impl!(Html, String);
}

/*
pub fn create_template_result<'cx>(ecx: &'cx mut ExtCtxt<'cx>, w_ident: ast::Ident, template: &Template) -> Box<MacResult + 'cx> {
    use self::output_item_writer::{WriteOutputItems, OutputItemWrite};
    use self::lang::{Lang, Html, Js};

    let mut items: Vec<P<ast::Item>> = vec![];
    template.write_output_items(ecx, &mut items as &mut OutputItemWrite<Html>);
    MacEager::items(SmallVector::many(items))
}
*/
