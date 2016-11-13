
use super::element_node::Element;
use simple_expr::SimpleExpr;


/// Represents a parsed content node
#[derive(Clone, Debug)]
pub enum ContentNode {
    ElementNode(Element),
    ExprNode(SimpleExpr),
    LiteralNode(LitValue),
}

/// Literal (static) value.
/// This value may be cached, compiled, interned, or otherwise statically stored, including
/// in cached javascript or html.
#[derive(Clone, Debug)]
pub enum LitValue {
    LitString(String)
}

pub mod parse {
    use super::{ContentNode, LitValue};
    use syntax::tokenstream::TokenTree;
    use syntax::codemap::Span;
    use syntax::ext::base::ExtCtxt;
    use syntax::parse::{token, PResult};
    use syntax::parse::parser::Parser;
    use nodes::element_node::parse::parse_element;

    use simple_expr::SimpleExprToken;
    use simple_expr::parse::parse_simple_expr;

    #[derive(Clone, Debug)]
    pub enum NodeType {
        Root,
        Named(String)
    }

    fn lit_string(tokens: &[SimpleExprToken]) -> Option<String> {
        let mut out = String::new();
        for token in tokens {
            if let &SimpleExprToken::LitString(ref contents) = token {
                   out.push_str(contents);
                   continue;
            };

            return None;
        }
        return Some(out)
    }

    fn parse_simple_expr_or_lit_node<'cx, 'a>(ecx: &'cx ExtCtxt, mut parser: &mut Parser<'a>, span: Span) -> PResult<'a, ContentNode> {
        try!(parser.expect(&token::OpenDelim(token::Brace)));
        let simple_expr = try!(parse_simple_expr(ecx, &mut parser, span, token::Brace));
        {
            let tokens = &simple_expr.tokens();
            let lit_string = lit_string(tokens);

            if let Some(contents) = lit_string {
                return Ok(ContentNode::LiteralNode(LitValue::LitString(contents)));
            }
        }

        Ok(ContentNode::ExprNode(simple_expr))
    }

    pub fn parse_contents<'cx, 'a>(ecx: &'cx ExtCtxt, mut parser: &mut Parser<'a>, span: Span, node_type: &NodeType) -> PResult<'a, Vec<ContentNode>> {
        let mut nodes: Vec<ContentNode> = Vec::new();

        loop {
            ecx.span_warn(span, &format!("Parsing contents ({:?}) - token: {:?}", &node_type, &parser.token));

            match parser.token {
                token::CloseDelim(token::Bracket) => {
                    ecx.span_warn(span, &format!("Parsing contents ({:?}) - complete", &node_type));
                    break;
                },

                token::Ident(_) => {
                    let element = try!(parse_element(ecx, parser, span, node_type));
                    nodes.push(ContentNode::ElementNode(element));
                },

                token::OpenDelim(token::Brace) => {
                    // Start of expression, which can be a literal value
                    ecx.span_warn(span, "Parsing contents - got open expression");
                    let node = try!(parse_simple_expr_or_lit_node(ecx, &mut parser, span));
                    ecx.span_warn(span, &format!("Parsing contents - expression: {:?}", &node));
                    nodes.push(node);
                },

                _ => {
                    ecx.span_err(span, &format!("Parsing contents ({:?}) - unknown token: {:?}", &node_type, &parser.token));
                }
            }
            parser.bump();
        }

        Ok(nodes)
    }
}

pub mod output_ast {
    use super::ContentNode;
    use output_actions::{OutputAction, IntoOutputActions, WriteOutputActions, OutputActionWrite};

    impl IntoOutputActions for ContentNode {
        fn into_output_actions(&self) -> Vec<OutputAction> {
            match self {
                &ContentNode::ElementNode(ref element) => element.into_output_actions(),
                &ContentNode::LiteralNode(ref lit) => lit.into_output_actions(),
                &ContentNode::ExprNode(ref simple_expr) => {
                    // TODO: Return a WriteResult serializing simple_expr
                    vec![OutputAction::WriteResult(simple_expr.clone())]
                }
            }
        }
    }

    impl WriteOutputActions for ContentNode {
        fn write_output_actions(&self, w: &mut OutputActionWrite) {
            match self {
                &ContentNode::ElementNode(ref element) => {
                    element.write_output_actions(w);
                },

                &ContentNode::LiteralNode(ref lit) => {
                    lit.write_output_actions(w);
                },

                &ContentNode::ExprNode(ref simple_expr) => {
                    // TODO: Write a WriteResult serializing simple_expr
                    w.write_output_action(&OutputAction::WriteResult(simple_expr.clone()));
                }
            };
        }
    }
}

pub mod output_ast_literal {
    use super::LitValue;
    use syntax::tokenstream::TokenTree;
    use syntax::ext::base::ExtCtxt;
    use output_actions::{OutputAction, IntoOutputActions, WriteOutputActions, OutputActionWrite};

    impl IntoOutputActions for LitValue {
        fn into_output_actions(&self) -> Vec<OutputAction> {
            match self {
                &LitValue::LitString(ref contents) => {
                    vec![OutputAction::Write(contents.to_owned())]
                }
            }
        }
    }

    impl WriteOutputActions for LitValue {
        fn write_output_actions(&self, w: &mut OutputActionWrite) {
            match self {
                &LitValue::LitString(ref contents) => {
                    w.write_output_action(&OutputAction::Write(contents.to_owned()));
                }
            }
        }
    }
}

pub mod js_write {
    // TODO
}
