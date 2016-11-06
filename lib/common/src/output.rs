

use std::rc::Rc;
use std::marker::PhantomData;

use syntax::ast;
use syntax::codemap::{Span, DUMMY_SP};
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult, SyntaxExtension, NormalTT, TTMacroExpander};
use syntax::ext::hygiene::Mark;
use syntax::tokenstream::TokenTree;
use syntax::util::small_vector::SmallVector;
use syntax::parse::PResult;
use syntax::parse::parser::Parser;
use syntax::ptr::P;

use itertools::Itertools;

use nodes;
use nodes::view_node::View;
use nodes::template_node::Template;

use codegen;
use codegen::lang::{Lang, Html, Js};
use codegen::output_string_writer::WriteOutputStrings;


fn define_ext<'cx>(ecx: &'cx mut ExtCtxt, name: &str, ext: Rc<SyntaxExtension>) {
    let ident = ecx.ident_of(name);
    (*ecx.resolver).add_ext(ident, ext);
}

macro_rules! define_named_template {
    ($ecx: ident, $name: ident, $ty:ident, $lang: expr, $item: ty, $template:expr) => ({
        let name = $name.name.to_string();
        let ext_name = &format!("emit_{}_view_{}", $lang, name);
        let ext = LangSyntaxExt::<$ty, $item>::with_data($ecx, &name, $template);
        define_ext($ecx, ext_name, Rc::new(NormalTT(Box::new(ext), None, true)));
    })
}

trait WriteBlockFactory {
    fn create_write_block<'cx>(&self, ecx: &'cx ExtCtxt<'cx>, w_ident: ast::Ident) -> Box<MacResult + 'cx>;
}

pub trait ToData<D, L: Lang> {
    fn to_data<'cx>(&self, &'cx ExtCtxt) -> D;
}

impl<L: Lang, S: WriteOutputStrings<L>> ToData<String, L> for S {
    fn to_data<'cx>(&self, ecx: &'cx ExtCtxt) -> String {
        let mut out = String::new();
        self.write_output_strings(ecx, &mut out);
        out
    }
}

impl <L: Lang, D> LangSyntaxExt<L, D> {
    fn with_data<'cx, S: ToData<D, L>>(ecx: &'cx ExtCtxt, name: &str, src: &S) -> LangSyntaxExt<L, D> {
        let data: D = src.to_data(ecx);

        LangSyntaxExt {
            name: name.to_owned(),
            data: data,
            _l: PhantomData
        }
    }
}

struct LangSyntaxExt<L: Lang, D> {
    name: String,
    data: D,
    _l: PhantomData<L>
}

/// Define TTMacroExpander for the given language, used the WriteBlockFactory trait above.
macro_rules! lang_expander {
    ($lang: ty, $item: ty) => (
        impl TTMacroExpander for LangSyntaxExt<$lang, $item> {
            fn expand<'cx>(&self, ecx: &'cx mut ExtCtxt, _: Span, tts: &[TokenTree]) -> Box<MacResult + 'cx> {
                let mut items: Vec<P<ast::Item>> = vec![];
                //let template = &self.decl.template;
                //template.write_items(ecx, &mut items);
                MacEager::items(SmallVector::many(items))
            }
        }
    )
}
macro_rules! lang_expander_empty {
    ($lang: ty, $item: ty) => (
        impl TTMacroExpander for LangSyntaxExt<$lang, $item> {
            fn expand<'cx>(&self, _: &'cx mut ExtCtxt, _: Span, _: &[TokenTree]) -> Box<MacResult + 'cx> {
                MacEager::items(SmallVector::zero())
            }
        }
    )
}

lang_expander!(Html, String);
lang_expander_empty!(Js, String);
//lang_expander!(Js);

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

/// Macro implementation: create a set of macros of the form emit_$lang_view_$template!($output_var);
/// which will render the parsed template in the given language.
pub fn expand_define_template<'cx>(ecx: &'cx mut ExtCtxt, span: Span, ident: ast::Ident, tts: Vec<TokenTree>) -> Box<MacResult + 'cx> {
    let mut parser = ecx.new_parser_from_tts(&tts);

    let name = ident.to_string().to_owned();
    match nodes::template_node::parse::parse_template(ecx, &mut parser, DUMMY_SP, &name) {
        Ok(template) => {
            define_named_template!(ecx, ident, Html, "html", String, &template);
            define_named_template!(ecx, ident, Js, "js", String, &template);
        },
        Err(err) => {
            ecx.span_fatal(span, &format!("Parsing failed for template {:?}: {:?}", &ident, &err));
        }
    };
    MacEager::items(SmallVector::zero())
}
