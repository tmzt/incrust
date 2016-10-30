
use syntax::codemap::{Span, DUMMY_SP};

use super::content_node::ContentNode;


#[derive(Clone, Debug)]
pub struct Element {
    element_type: String,
    span: Span,
    nodes: Vec<ContentNode>,
}

pub mod parse {
    use super::Element;
    use syntax::tokenstream::TokenTree;
    use syntax::codemap::{Span, DUMMY_SP};
    use syntax::ext::base::ExtCtxt;
    use syntax::parse::{token, PResult};
    use syntax::parse::parser::Parser;

    use output_actions::{OutputAction, IntoOutputActions};
    use simple_expr::SimpleExpr;
    use simple_expr::parse::parse_simple_expr;
    use nodes::content_node::parse::{NodeType, parse_contents};

    pub fn parse_element<'cx, 'a>(ecx: &'cx ExtCtxt, mut parser: &mut Parser<'a>, span: Span, node_type: &NodeType) -> PResult<'a, Element> {
        let element_type_token = try!(parser.parse_ident());
        let element_type = element_type_token.name.to_string().to_owned();

        ecx.span_warn(span, &format!("Parsing contents ({:?}) - got element type: {:?}", node_type, &element_type));

        try!(parser.expect(&token::OpenDelim(token::Bracket)));

        let nodes = try!(parse_contents(ecx, &mut parser, span, &NodeType::Named(element_type.to_owned())));

        Ok(Element {
            element_type: element_type.to_owned(),
            span: span,
            nodes: nodes,
        })
    }
}

pub mod output {
    use super::Element;
    use syntax::ext::base::ExtCtxt;
    use output_actions::{OutputAction, IntoOutputActions};

    impl IntoOutputActions for Element {
        fn into_output_actions<'cx>(&self, ecx: &'cx ExtCtxt) -> Vec<OutputAction> {
            let nodes = &self.nodes;
            let element_type = &self.element_type;
            let mut output_actions = Vec::new();

            output_actions.push(OutputAction::WriteOpen(element_type.clone()));

            let child_actions: Vec<OutputAction> = nodes.iter()
                .flat_map(|node| node.into_output_actions(ecx))
                .collect();
            output_actions.extend(child_actions);

            output_actions.push(OutputAction::WriteClose(element_type.clone()));

            output_actions
        }
    }
}
