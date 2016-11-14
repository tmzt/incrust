

pub mod expander {
    use syntax::ast;
    use syntax::ext::base::{ExtCtxt, MacResult, DummyResult, MacEager};
    use syntax::ext::build::AstBuilder;
    use syntax::parse::parser::Parser;
    use syntax::parse::{token, PResult};
    use syntax::tokenstream::TokenTree;
    use syntax::util::small_vector::SmallVector;
    use syntax::codemap::{DUMMY_SP, Span};
    use syntax::ptr::P;

    use incrust_common::codegen::lang::{Html, Js};
    use incrust_common::nodes::template_node::parse::parse_template;

    use incrust_common::codegen::output_item_writer::IntoOutputItem;
    use incrust_common::codegen::named_output::NamedOutput;

    pub enum RenderLang {
        RenderHtml,
        RenderJs
    }

    fn process_contents<'cx, 'r>(ecx: &'cx mut ExtCtxt<'r>, span: Span, ident: ast::Ident, mut parser: &mut Parser<'r>) -> Box<MacResult + 'cx> {
        let template_name = ident.name.to_string();
        ecx.span_warn(span, &format!("Parsing contents of template {}", &template_name));

        macro_rules! define_lang_outputs (
            ($ecx: expr, $template: ident, $template_name: expr, $lang: ident) => ({
                let sources: Vec<P<ast::Item>> = $template.nodes().iter().map(|node| {
                    $ecx.span_warn(DUMMY_SP, &format!("Source: {:?}", &node));
                    let lang_node: &IntoOutputItem<$lang> = node;
                    let item = lang_node.into_output_item($ecx, (node as &NamedOutput<$lang>).output_name());
                    $ecx.span_warn(DUMMY_SP, &format!("Source item: {:?}", &item));
                    item
                }).collect();
                sources
            })
        );

        match parse_template(ecx, parser, span, &template_name) {
            Ok(template) => {
                ecx.span_warn(span, &format!("Parsed template: {:?}", &template));

                let mut items = Vec::new();
                items.append(&mut define_lang_outputs!(ecx, template, "main", Html));
                items.append(&mut define_lang_outputs!(ecx, template, "main", Js));
                MacEager::items(SmallVector::many(items))
            },

            Err(mut err) => {
                err.emit();
                DummyResult::expr(span)
            }
        }
    }

    fn process_render<'cx, 'r>(ecx: &'cx ExtCtxt, span: Span, mut parser: &mut Parser<'r>) -> PResult<'r, Box<MacResult + 'cx>> {
        let html_writer = try!(parser.parse_expr());
        try!(parser.expect(&token::Comma));
        let js_writer = try!(parser.parse_expr());
        try!(parser.expect(&token::Comma));
        let template_name = try!(parser.parse_ident()).to_string();
        try!(parser.expect(&token::Comma));
        let output_ty = try!(parser.parse_ident()).to_string();
        try!(parser.expect(&token::Comma));
        let output_name = try!(parser.parse_ident()).to_string();
        try!(parser.expect(&token::Comma));
        let lang = try!(parser.parse_ident());

        match output_ty {
            _ if output_ty == "view" => (),
            _ if output_ty == "store" => (),
            _ => {
                ecx.span_fatal(span, &format!("Unsupported output type."));
            }
        };

        let lang_str = lang.to_string().to_lowercase();
        match &lang_str {
            _ if lang_str == "html" => RenderLang::RenderHtml,
            _ if lang_str == "js" => RenderLang::RenderJs,
            _ => {
                ecx.span_fatal(span, &format!("Unsupported render language."));
            }
        };

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