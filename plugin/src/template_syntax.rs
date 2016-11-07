

pub mod expander {
    use std::marker::PhantomData;
    use syntax::ast;
    use syntax::ext::base::{ExtCtxt, SyntaxExtension, NormalTT, TTMacroExpander, MacResult, DummyResult, MacEager};
    use syntax::ext::build::AstBuilder;
    use syntax::parse::parser::Parser;
    use syntax::parse::{token, PResult};
    use syntax::tokenstream::TokenTree;
    use syntax::util::small_vector::SmallVector;
    use syntax::codemap::{DUMMY_SP, Span};
    use syntax::ptr::P;

    use incrust_common::codegen::lang::{Lang, Html, Js};
    use incrust_common::codegen::output_item_writer::{WriteOutputItems, OutputItemWrite};
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

    /*
    fn define_lang_outputs<'cx, L: Lang>(ecx: &'cx ExtCtxt, template: &Template, template_name: &str, lang: &str) -> Box<MacResult + 'cx> {
        let mut items: Vec<P<ast::Item>> = vec![];
        template.write_output_items(ecx, &mut items as &mut OutputItemWrite<Html>);

        /*
        let stmts: Vec<ast::Stmt> = items.iter().map(|item| ecx.stmt_item(DUMMY_SP, item.to_owned())).collect();
        let block = ecx.block(DUMMY_SP, stmts);

        let name = ecx.ident_of(&format!("rusttemplate_render_template_{}_view_{}_{}", template_name, &"root", lang));

        let item = {
            let args = vec![
                ecx.arg(DUMMY_SP, ecx.ident_of("html_writer"), quote_ty!(ecx, &mut String)),
                ecx.arg(DUMMY_SP, ecx.ident_of("js_writer"), quote_ty!(ecx, &mut String)),
            ];
            let ret_ty = quote_ty!(ecx, ());
            ecx.item_fn(DUMMY_SP, name, args, ret_ty, block)
        };

        MacEager::items(SmallVector::one(item))
        */

        let item = (template as &IntoOutputItem<Html>).into_output_item(ecx, &"root");
        MacEager::items(SmallVector::one(item))
    }
    */

    use incrust_common::codegen::output_item_writer::IntoOutputItem;
    use incrust_common::codegen::named_output::{NamedOutput, NamedOutputType};

    pub enum RenderLang {
        RenderHtml,
        RenderJs
    }

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

        macro_rules! define_lang_outputs (
            ($ecx: expr, $template: ident, $template_name: expr, $lang: ident) => ({
                let lang = stringify!($lang).to_lowercase();
                let nodes = $template.nodes();
                let sources: Vec<P<ast::Item>> = nodes.iter().map(|node| {
                    $ecx.span_warn(DUMMY_SP, &format!("Source: {:?}", &node));
                    let lang_node: &IntoOutputItem<$lang> = node;
                    let item = lang_node.into_output_item($ecx, (node as &NamedOutput<$lang>).output_name());
                    $ecx.span_warn(DUMMY_SP, &format!("Source item: {:?}", &item));
                    item
                }).collect();
                sources

                //let tmpl: &IntoOutputItem<$lang> = &$template;
                //tmpl.into_output_item(ecx, &"root")
            })
        );

        match parse_template(ecx, &mut parser, span, &template_name) {
            Ok(template) => {
                ecx.span_warn(span, &format!("Parsed template: {:?}", &template));

                let mut items = Vec::new();
                items.append(&mut define_lang_outputs!(ecx, template, "main", Html));
                items.append(&mut define_lang_outputs!(ecx, template, "main", Js));

                //MacEager::items(SmallVector::many(items))
                // Empty (but must consist of items)                                                                                                    
                //MacEager::items(SmallVector::zero())
                MacEager::items(SmallVector::many(items))
            },

            Err(mut err) => {
                err.emit();
                DummyResult::expr(span)
            }
        }
    }

    fn process_render<'cx, 'r>(ecx: &'cx ExtCtxt, span: Span, mut parser: &mut Parser<'r>) -> PResult<'r, Box<MacResult + 'cx>> {
        let html_writer = try!(parser.parse_ident());
        parser.expect(&token::Comma);
        let js_writer = try!(parser.parse_ident());
        parser.expect(&token::Comma);
        let template_name = try!(parser.parse_ident()).to_string();
        parser.expect(&token::Comma);
        let output_ty = try!(parser.parse_ident()).to_string();
        parser.expect(&token::Comma);
        let output_name = try!(parser.parse_ident()).to_string();
        parser.expect(&token::Comma);
        let lang = try!(parser.parse_ident());

        match output_ty {
            _ if output_ty == "view" => (),
            _ if output_ty == "store" => (),
            _ => {
                ecx.span_fatal(span, &format!("Unsupported output type."));
            }
        };

        let lang_str = lang.to_string().to_lowercase();
        let render_lang = match &lang_str {
            _ if lang_str == "html" => RenderLang::RenderHtml,
            _ if lang_str == "js" => RenderLang::RenderJs,
            _ => {
                ecx.span_fatal(span, &format!("Unsupported render language."));
            }
        };

        // rusttemplate_render_template_main_store_counter_js
        // rusttemplate_render_template_main_view_root_js

        // example: rusttemplate_render_template_main_view_root_html
        // example: rusttemplate_render_template_main_store_counter_js
        let render_ident = ecx.ident_of(&format!("rusttemplate_render_template_{}_{}_{}_{}", &template_name, &output_ty, &output_name, &lang_str));
        let expr = quote_expr!(ecx, {
            $render_ident($html_writer, $js_writer);
        });
        let stmt = ecx.stmt_expr(expr);

        Ok(MacEager::stmts(SmallVector::one(stmt)))
    }

    /// Macro implementation: create a set of macros of the form emit_$lang_view_$template!($output_var);
    /// which will render the parsed template in the given language.
    pub fn expand_template<'cx, 'r>(ecx: &'cx mut ExtCtxt<'r>, span: Span, ident: ast::Ident, tts: Vec<TokenTree>) -> Box<MacResult + 'cx> {
        let mut parser = ecx.new_parser_from_tts(&tts);
        process_contents(ecx, span, ident, &mut parser)
    }

    /// Macro implementation: render named output in template, with output name
    /// ($html_writer: ident, $js_writer: ident, $output_ty: ident, $template_name: ident, $output_name: ident, $render_lang: ident)
    pub fn expand_render_output<'cx, 'r>(ecx: &'cx mut ExtCtxt<'r>, span: Span, tts: &[TokenTree]) -> Box<MacResult + 'cx> {
        let mut parser = ecx.new_parser_from_tts(&tts);
        match process_render(ecx, span, &mut parser) {
            Ok(result) => {
                result
            },
            Err(err) => {
                ecx.span_fatal(span, &format!("Error rendering template: {:?}", err));
                DummyResult::any(span)
            }
        }
    }

}