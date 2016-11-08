
use syntax::tokenstream::TokenTree;
use syntax::codemap::{Span, DUMMY_SP};
use syntax::ext::base::ExtCtxt;
use syntax::ext::quote::rt::ToTokens;
use syntax::parse::{token, PResult};
use syntax::parse::parser::Parser;

use output_actions::{OutputAction, IntoOutputActions};
use simple_expr::SimpleExpr;
use simple_expr::parse::parse_simple_expr;

use nodes::content_node::ContentNode;

/// Represents a parsed view in template contents
#[derive(Clone, Debug)]
pub struct View {
    name: String,
    span: Span,
    nodes: Vec<ContentNode>
}

impl View {
    pub fn name(&self) -> &str {
        &self.name
    }
}

pub mod parse {
    use super::View;
    use syntax::tokenstream::TokenTree;
    use syntax::codemap::{Span, DUMMY_SP};
    use syntax::ext::base::ExtCtxt;
    use syntax::parse::{token, PResult};
    use syntax::parse::parser::Parser;
    use nodes::content_node::parse::{NodeType, parse_contents};

    pub fn parse_view<'cx, 'a>(ecx: &'cx ExtCtxt, mut parser: &mut Parser<'a>, span: Span) -> PResult<'a, View> {
        //let view_token = try!(parser.parse_ident());
        let view_name = try!(parser.parse_ident());

        try!(parser.expect(&token::OpenDelim(token::Bracket)));

        let nodes = try!(parse_contents(ecx, parser, span, &NodeType::Root));

        Ok(View {
            name: view_name.name.to_string(),
            span: span,
            nodes: nodes,
        })
    }
}

mod output {
    use super::View;
    use syntax::ext::base::ExtCtxt;
    use output_actions::{OutputAction, IntoOutputActions, WriteOutputActions, OutputActionWrite};
    use js_write::{WriteJsFunctions, JsWriteFunctions, WriteJs};

    impl IntoOutputActions for View {
        fn into_output_actions<'cx>(&self) -> Vec<OutputAction> {
            let name = &self.name;
            let nodes = &self.nodes;

            let output_actions: Vec<OutputAction> = nodes.iter()
                .flat_map(|node| node.into_output_actions())
                .collect();

            output_actions
        }
    }

    impl WriteOutputActions for View {
        fn write_output_actions(&self, w: &mut OutputActionWrite) {
            for node in &self.nodes {
                node.write_output_actions(w);
            }
        }
    }

    impl WriteJsFunctions for View {
        fn write_js_functions(&self, funcs: &mut JsWriteFunctions) {
            let view_name = self.name();
            let func_name = format!("rusttemplate_render_template_{}_view_{}_calls", "main", &view_name);

            let mut output_actions = Vec::new();
            self.write_output_actions(&mut output_actions);

            // TODO: Generate the argument list
            funcs.function(&func_name, vec!["counter"], &|js| {
                output_actions.write_js(js);
            });
        }
    }
}
