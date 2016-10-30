
use syntax::codemap::{Span, DUMMY_SP};

use super::element_node::Element;
use simple_expr::SimpleExpr;

/// Represents a parsed content node
#[derive(Clone, Debug)]
pub enum ContentNode {
    ElementNode(Element),
    ExprNode(SimpleExpr),
    //LiteralNode(TemplateLiteral),
}

/// Represents a static literal value within the contents of a view or element.
#[derive(Clone, Debug)]
pub struct ContentLiteral {
    span: Span,
    val: LitValue
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
    use syntax::codemap::{Span, DUMMY_SP};
    use syntax::ext::base::ExtCtxt;
    use syntax::ext::quote::rt::ToTokens;
    use syntax::parse::{token, PResult};
    use syntax::parse::parser::Parser;

    use output_actions::{OutputAction, IntoOutputActions};
    use simple_expr::SimpleExpr;
    use simple_expr::parse::parse_simple_expr;

    fn parse_expr_node<'cx, 'a>(ecx: &'cx ExtCtxt, mut parser: &mut Parser<'a>, span: Span) -> PResult<'a, ContentNode> {
        let simple_expr = try!(parse_simple_expr(ecx, &mut parser, span));
        Ok(ContentNode::ExprNode(simple_expr))
    }

    pub fn parse_contents<'cx, 'a>(ecx: &'cx ExtCtxt, mut parser: &mut Parser<'a>, span: Span) -> PResult<'a, Vec<ContentNode>> {
        let mut nodes: Vec<ContentNode> = Vec::new();

        loop {
            match parser.token {
                token::Ident(_) => {
                    if let token::Ident(ref ident) = parser.token {
                        let element_type = ident.name.to_string().to_owned();
                        ecx.span_warn(span, &format!("Parsing contents - got element type: {:?}", &element_type));

                        // TODO: Parse element
                    }
                },

                token::OpenDelim(token::Brace) => {
                    // Start of expression, which can be a literal value
                    // NEXTREV: Handle literal differently, so it can be statically compiled
                    ecx.span_warn(span, "Parsing contents - got open expression");
                    let node = try!(parse_expr_node(ecx, &mut parser, span));
                    nodes.push(node);
                    break;
                },

                token::CloseDelim(token::Bracket) => {
                    ecx.span_warn(span, "Parsing contents - complete");
                    break;
                },

                _ => {
                    ecx.span_err(span, &format!("Parsing contents - unknown token: {:?}", &parser.token));
                }
            }
        }

        Ok(nodes)
    }
}

pub mod output_ast {
    use super::ContentNode;
    use output_actions::{OutputAction, IntoOutputActions};
    use syntax::ext::base::ExtCtxt;

    impl IntoOutputActions for ContentNode {
        fn into_output_actions<'cx>(&self, ecx: &'cx ExtCtxt) -> Vec<OutputAction> {
            match self {
                &ContentNode::ElementNode(ref element) => element.into_output_actions(ecx),
                &ContentNode::ExprNode(ref simple_expr) => simple_expr.into_output_actions(ecx)
            }
        }
    }
}

pub mod output_ast_literal {
    use super::{ContentNode, ContentLiteral, LitValue};
    use syntax::tokenstream::TokenTree;
    use syntax::codemap::{Span, DUMMY_SP};
    use syntax::ext::base::ExtCtxt;
    use syntax::ext::quote::rt::ToTokens;

    impl ToTokens for ContentLiteral {
        fn to_tokens(&self, ecx: &ExtCtxt) -> Vec<TokenTree> {
            Vec::from(self.val.to_tokens(ecx).as_slice())
        }
    }

    impl ToTokens for LitValue {
        fn to_tokens(&self, ecx: &ExtCtxt) -> Vec<TokenTree> {
            let val = match *self {
                LitValue::LitString(ref contents) => {
                    let s = quote_expr!(ecx, $contents.to_owned());
                    quote_expr!(ecx, OutputAction::Write($s))
                }
            };

            val.to_tokens(ecx)
        }
    }
}

pub mod js_write {
    // TODO
}
