

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
use syntax::ptr::P;

use itertools::Itertools;

use nodes::view_node::View;
use nodes::template_node::Template;

use codegen;
use codegen::item_writer::WriteItems;

//use incrust_common::codegen;
//use incrust_common::jsgen::IntoJsFunction;
//use incrust_common::output_actions::IntoOutputActions;
//use incrust_common::view::parse_view;
//use incrust_common::compiled_view::CompiledView;


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

#[derive(Debug, Clone)]
struct NamedTemplateDecl {
    name: String,
    template: Template
}

enum Rust {}
enum Js {}

trait WriteBlockFactory {
    fn create_write_block<'cx>(&self, ecx: &'cx ExtCtxt<'cx>, w_ident: ast::Ident) -> Box<MacResult + 'cx>;
}

trait Lang {
    fn ext() -> &'static str;
}

impl <L: Lang> LangSyntaxExt<L> {
    fn for_template(name: &str, template: Template) -> LangSyntaxExt<L> {
        let lang_ext = LangSyntaxExt {
            decl: NamedTemplateDecl { name: String::from(name), template: template },
            _l: PhantomData
        };

        lang_ext
    }
}

#[derive(Debug, Clone)]
struct LangSyntaxExt<L: Lang> {
    decl: NamedTemplateDecl,
    _l: PhantomData<L>
}

/// Define TTMacroExpander for the given language, used the WriteBlockFactory trait above.
macro_rules! lang_expander {
    ($lang: ty) => (
        impl TTMacroExpander for LangSyntaxExt<$lang> {
            fn expand<'cx>(&self, ecx: &'cx mut ExtCtxt, _: Span, tts: &[TokenTree]) -> Box<MacResult + 'cx> {
                /*
               // self.decl.expand(ecx, span, tts)
                let mut parser = ecx.new_parser_from_tts(tts);
                let w_ident = parser.parse_ident().unwrap();
                //self.create_write_block(ecx, w_ident)
                MacEager::items(SmallVector::zero())
                */
                let mut items: Vec<P<ast::Item>> = vec![];
                let template = &self.decl.template;
                template.write_items(ecx, &mut items);
                MacEager::items(SmallVector::many(items))
            }
        }
    )
}
lang_expander!(Rust);
//lang_expander!(Js);

macro_rules! lang {
    ($lang: ty, $ext: expr) => (
        impl Lang for $lang {
            fn ext() -> &'static str { $ext }
        }
    )
}
lang!(Rust, "rust");
lang!(Js, "js");

/*
impl WriteBlockFactory for LangSyntaxExt<Rust> {
    fn create_write_block<'cx>(&self, ecx: &'cx ExtCtxt<'cx>, w_ident: ast::Ident) -> Box<MacResult + 'cx> {
        //codegen::create_template_write_block(ecx, w_ident, &self.decl.views)
        codegen::create_template_result(ecx, w_ident, &self.decl.template)
    }
}
*/

/*
impl WriteBlockFactory for LangSyntaxExt<Js> {
    fn create_write_block<'cx>(&self, ecx: &'cx mut ExtCtxt, w_ident: ast::Ident) -> Box<MacResult + 'cx> {
        /*
        let funcs: Vec<String> = self.decl.views.iter()
            .map(|view| view.into_js_function(ecx))
            .intersperse("; ".into())
            .collect();
        */
        let funcs: Vec<String> = vec![];

        codegen::create_write_statements_block(ecx, w_ident, funcs.as_slice())MacResult
    }
}
*/

/*
impl TTMacroExpander for NamedTemplateDecl {
    fn expand<'cx>(&self, ecx: &'cx mut ExtCtxt, _: Span, tts: &[TokenTree]) -> Box<MacResult + 'cx> {
        let mut parser = ecx.new_parser_from_tts(tts);
        let w_ident = parser.parse_ident().unwrap();
        //codegen::create_template_write_block(ecx, w_ident, &self.compiled_views);
        MacEager::items(SmallVector::zero())
    }
}
*/

/*
impl TTMacroExpander for NamedTemplateDecl {
    fn expand<'cx>(&self, ecx: &'cx mut ExtCtxt<'cx>, _: Span, tts: &[TokenTree]) -> Box<MacResult + 'cx> {
        let mut parser = ecx.new_parser_from_tts(tts);
        let w_ident = parser.parse_ident().unwrap();
        //codegen::create_template_write_block(ecx, w_ident, &self.views)
        //codegen::create_template_result(ecx, w_ident, self.template)
        //codegen::create_template_result(ecx, w_ident, self.template);
        MacEager::items(SmallVector::zero())
    }
}
*/

/*
fn parse_template_into_compiled_view<'cx, 'a>(ecx: &'cx mut ExtCtxt, span: Span, parser: &mut Parser<'a>)
                                                                            -> PResult<'a, View> {
    let view = try!(parse_view(ecx, parser, span));
    let output_actions = view.into_output_actions(ecx);
    ecx.span_warn(span, &format!("output_actions: {:?}", output_actions));

    let name = String::from(view.name());
    Ok(View::from_output_actions(name, output_actions))
}

/// Macro implementation: create a set of macros of the form emit_$lang_view_$template!($output_var);
/// which will render the parsed template in the given language.
pub fn expand_define_template<'cx>(ecx: &'cx mut ExtCtxt, span: Span, ident: ast::Ident, tts: Vec<TokenTree>) -> Box<MacResult + 'cx> {
    let mut parser = ecx.new_parser_from_tts(&tts);

    match parse_template_into_compiled_view(ecx, span, &mut parser) {
        Ok(compiled_view) => {
            let compiled_views = vec![compiled_view];
            define_named_template!(ecx, ident, Rust, "rust", compiled_views);
            define_named_template!(ecx, ident, Js, "js", compiled_views);

            // Empty (but must consist of items)
            MacEager::items(SmallVector::zero())
        },

        Err(mut err) => {
            err.emit();
            DummyResult::expr(span)
        }
    }

}
*/
