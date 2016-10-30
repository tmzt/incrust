

use std::rc::Rc;
use std::marker::PhantomData;

use syntax::ast;
use syntax::codemap::Span;
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult, SyntaxExtension, NormalTT, TTMacroExpander};
use syntax::ext::hygiene::Mark;
use syntax::tokenstream::TokenTree;
use syntax::util::small_vector::SmallVector;
use syntax::parse::PResult;
use syntax::parse::parser::Parser;

use itertools::Itertools;

use incrust_common::nodes::*;

use incrust_common::codegen;
use incrust_common::js_write::{WriteJs, JsWrite};
use incrust_common::output_actions::IntoOutputActions;


/*
fn define_ext<'cx>(ecx: &'cx mut ExtCtxt, name: &str, ext: Rc<SyntaxExtension>) {
    let ident = ecx.ident_of(name);
    // TODO: This is changed to add_ext in b4906a (https://github.com/rust-lang/rust/commit/b4906a93)
    // update when updating nightly
    (*ecx.resolver).add_macro(Mark::root(), ident, ext);
}
*/

fn define_ext<'cx>(ecx: &'cx mut ExtCtxt, name: &str, ext: Rc<SyntaxExtension>) {
    let ident = ecx.ident_of(name);
    (*ecx.resolver).add_ext(ident, ext);
}

macro_rules! define_named_template {
    ($ecx: ident, $name: ident, $ty:ident, $lang: expr, $views:expr) => ({
        let name = $name.name.to_string();
        let ext_name = &format!("emit_{}_view_{}", $lang, name);
        let ext = LangSyntaxExt::<$ty>::create_template(&name, $views.clone());
        define_ext($ecx, ext_name, Rc::new(NormalTT(Box::new(ext), None, true)));
    })
}


trait WriteBlockFactory {
    fn create_write_block<'cx>(&self, ecx: &'cx mut ExtCtxt, w_ident: ast::Ident) -> Box<MacResult + 'cx>;
}

mod lang {
    pub enum Rust {}
    pub enum Js {}

    pub trait Lang {
        fn ext() -> &'static str;
    }
}

pub mod expander {
    use std::marker::PhantomData;
    use syntax::ast;
    use syntax::ext::base::{ExtCtxt, NormalTT, TTMacroExpander, MacResult, DummyResult, MacEager};
    use syntax::tokenstream::TokenTree;
    use syntax::util::small_vector::SmallVector;
    use syntax::codemap::Span;

    use super::lang::{Lang, Js, Rust};
    use incrust_common::nodes::*;
    use incrust_common::nodes::template_node::{Template, TemplateNode};

    #[derive(Debug, Clone)]
    struct LangSyntaxExt<L: Lang> {
        decl: Template,
        _l: PhantomData<L>
    }

    impl <L: Lang> LangSyntaxExt<L> {
        /*
        fn create_template(name: &str, compiled_views: Vec<CompiledView>) -> LangSyntaxExt<L> {
            let lang_ext = LangSyntaxExt {
                decl: Template { name: String::from(name), compiled_views: compiled_views },
                _l: PhantomData
            };

            lang_ext
        }
        */
    }

    /// Define TTMacroExpander for the given language, used the WriteBlockFactory trait above.
    macro_rules! lang_expander {
        ($lang: ty) => (
            /*
            use syntax::ext::base::{ExtCtxt, NormalTT, TTMacroExpander, MacResult};

            impl TTMacroExpander for LangSyntaxExt<$lang> {
                fn expand<'cx>(&self, ecx: &'cx mut ExtCtxt, _: Span, tts: &[TokenTree]) -> Box<MacResult + 'cx> {
                // self.decl.expand(ecx, span, tts)
                    let mut parser = ecx.new_parser_from_tts(tts);
                    let w_ident = parser.parse_ident().unwrap();
                    self.create_write_block(ecx, w_ident)
                }
            }
            */
        )
    }
    lang_expander!(Rust);
    lang_expander!(Js);

    macro_rules! lang {
        ($lang: ty, $ext: expr) => (
            impl Lang for $lang {
                fn ext() -> &'static str { $ext }
            }
        )
    }
    lang!(Rust, "rust");
    lang!(Js, "js");

    /// Macro implementation: create a set of macros of the form emit_$lang_view_$template!($output_var);
    /// which will render the parsed template in the given language.
    pub fn expand_define_template<'cx>(ecx: &'cx mut ExtCtxt, span: Span, ident: ast::Ident, tts: Vec<TokenTree>) -> Box<MacResult + 'cx> {
        use incrust_common::nodes::template_node::parse::parse_template;

        let mut parser = ecx.new_parser_from_tts(&tts);

        match parse_template(ecx, &mut parser, span) {
            Ok(template) => {
                // TODO: Implement

                // Empty (but must consist of items)
                MacEager::items(SmallVector::zero())
            },

            Err(mut err) => {
                err.emit();
                DummyResult::expr(span)
            }
        }

        /*
        match parse_template(ecx, &mut parser, span) {
            Ok(template) => {
                let views = template.views();
                define_named_template!(ecx, ident, Rust, "rust", compiled_views);
                define_named_template!(ecx, ident, Js, "js", compiled_views);

                // Empty (but must consist of items)
                MacEager::items(SmallVector::zero())
            }
        }
        */

        /*
        match parse_template_into_compiled_view(ecx, span, &mut parser) {
            Ok(compiled_view) => {
                // TODO: Implement

                // Empty (but must consist of items)
                MacEager::items(SmallVector::zero())            
            },

            Err(mut err) => {
                err.emit();
                DummyResult::expr(span)
            }
        }
        */

    }
}

/*
use lang::{Js, Rust};

impl WriteBlockFactory for LangSyntaxExt<Rust> {
    fn create_write_block<'cx>(&self, ecx: &'cx mut ExtCtxt, w_ident: ast::Ident) -> Box<MacResult + 'cx> {
        codegen::create_template_write_block(ecx, w_ident, &self.decl.compiled_views)
    }
}

impl WriteBlockFactory for LangSyntaxExt<Js> {
    fn create_write_block<'cx>(&self, ecx: &'cx mut ExtCtxt, w_ident: ast::Ident) -> Box<MacResult + 'cx> {
        let funcs: Vec<String> = self.decl.compiled_views.iter()
            .map(|compiled_view| {
                let mut out = String::new();
                compiled_view.write_js(&mut out);
                out
            })
            //.intersperse("; ".into())
            .collect();

        codegen::create_write_statements_block(ecx, w_ident, funcs.as_slice())
    }
}
*/