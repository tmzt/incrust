

pub mod expander {
    use std::marker::PhantomData;
    use syntax::ast;
    use syntax::ext::base::{ExtCtxt, NormalTT, TTMacroExpander, MacResult, DummyResult, MacEager};
    use syntax::parse::parser::Parser;
    use syntax::tokenstream::TokenTree;
    use syntax::util::small_vector::SmallVector;
    use syntax::codemap::Span;

    use incrust_common::codegen::WriteItems;
    use incrust_common::codegen::lang::{Lang, Html, Js};
    use incrust_common::nodes::*;
    use incrust_common::nodes::template_node::{Template, TemplateNode};
    use incrust_common::nodes::template_node::parse::parse_template;

    fn process_contents<'cx>(ecx: &'cx mut ExtCtxt, span: Span, ident: ast::Ident, mut parser: &mut Parser) -> Box<MacResult + 'cx> {
        let template_name = ident.name.to_string();
        ecx.span_warn(span, &format!("Parsing contents of template {}", &template_name));

        match parse_template(ecx, &mut parser, span, &template_name) {
            Ok(template) => {
                ecx.span_warn(span, &format!("Parsed template: {:?}", &template));

                // Emit items for each node in the template
                template.write_items(ecx);

                // Empty (but must consist of items)
                MacEager::items(SmallVector::zero())
            },

            Err(mut err) => {
                err.emit();
                DummyResult::expr(span)
            }
        }
    }

    /// Macro implementation: create a set of macros of the form emit_$lang_view_$template!($output_var);
    /// which will render the parsed template in the given language.
    pub fn expand_template<'cx>(ecx: &'cx mut ExtCtxt, span: Span, ident: ast::Ident, tts: Vec<TokenTree>) -> Box<MacResult + 'cx> {
        let mut parser = ecx.new_parser_from_tts(&tts);
        process_contents(ecx, span, ident, &mut parser)
    }
}