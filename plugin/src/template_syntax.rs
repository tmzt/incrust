

pub mod expander {
    use std::marker::PhantomData;
    use syntax::ast;
    use syntax::ext::base::{ExtCtxt, SyntaxExtension, NormalTT, TTMacroExpander, MacResult, DummyResult, MacEager};
    use syntax::parse::parser::Parser;
    use syntax::tokenstream::TokenTree;
    use syntax::util::small_vector::SmallVector;
    use syntax::codemap::Span;
    use syntax::ptr::P;

    use incrust_common::codegen::lang::{Lang, Html, Js};
    use incrust_common::codegen::item_writer::WriteItems;
    use incrust_common::codegen::named_output_writer::{WriteNamedOutputs, NamedOutputWrite};
    use incrust_common::output_actions::OutputAction;
    use incrust_common::nodes::*;
    use incrust_common::nodes::template_node::{Template, TemplateNode};
    use incrust_common::nodes::template_node::parse::parse_template;

    use std::rc::Rc;


    /*
    fn define_lang_outputs<'cx: 'r, 'r, L: Lang, D>(ecx: &'cx ExtCtxt<'r>, span: Span, source: &WriteNamedOutputs<L, D>) {
        let mut named_outputs: Vec<NamedOutputExt<L, D>> = vec![];
        source.write_named_outputs(ecx, &mut named_outputs);
    }
    */

    fn process_contents<'cx, 'r>(ecx: &'cx mut ExtCtxt<'r>, span: Span, ident: ast::Ident, mut parser: &mut Parser) -> Box<MacResult + 'cx> {
        let template_name = ident.name.to_string();
        ecx.span_warn(span, &format!("Parsing contents of template {}", &template_name));

        /*
        macro_rules! define_lang_outputs (
            ($ecx: ident, $source: expr, $w: expr, $lang: ident, $data_ty: ty) => ({
                let s = $source: &WriteNamedOutputs<$lang, $data_ty>;
                s.write_named_outputs($ecx, $w);
            })
        );
        */

        match parse_template(ecx, &mut parser, span, &template_name) {
            Ok(template) => {
                ecx.span_warn(span, &format!("Parsed template: {:?}", &template));

                // Emit items for each node in the template
                //template.write_items(ecx);

                // Emit named output (macros) for each node in the template, for each supported language
                //let mut named_outputs = vec![];
                //define_lang_outputs!(ecx, &template, &mut named_outputs, Html, String);
                //define_lang_outputs::<Html, String>(ecx, span, &template, &mut named_outputs);
                //define_lang_outputs::<Html, String>(ecx, span, &template);

                let name = format!("template_{}", template_name).to_owned();
                let data: Vec<OutputAction> = vec![];
                //define_named_output!(ecx, name, Html, Vec<OutputAction>, data);

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
    pub fn expand_template<'cx, 'r>(ecx: &'cx mut ExtCtxt<'r>, span: Span, ident: ast::Ident, tts: Vec<TokenTree>) -> Box<MacResult + 'cx> {
        let mut parser = ecx.new_parser_from_tts(&tts);
        process_contents(ecx, span, ident, &mut parser)
    }
}