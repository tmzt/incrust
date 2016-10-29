
use syntax::tokenstream::TokenTree;
use syntax::codemap::{Span, DUMMY_SP};
use syntax::ext::base::ExtCtxt;
use syntax::ext::quote::rt::ToTokens;
use syntax::parse::{token, PResult};
use syntax::parse::parser::Parser;

use output_actions::{OutputAction, IntoOutputActions};
use simple_expr::SimpleExpr;
use simple_expr::parse::parse_simple_expr;


#[derive(Clone, Debug)]
pub struct Element {
    element_type: String,
    span: Span,
    nodes: Vec<TemplateNode>,
}

#[derive(Clone, Debug)]
pub struct TemplateLiteral {
    span: Span,
    val: LitValue
}

#[derive(Clone, Debug)]
pub enum LitValue {
    LitString(String)
}

impl ToTokens for TemplateLiteral {
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

// Represents a parsed node in the template syntax
#[derive(Clone, Debug)]
pub enum TemplateNode {
    ElementNode(Element),
    ExprNode(SimpleExpr),
    LiteralNode(TemplateLiteral),
}

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

impl IntoOutputActions for TemplateLiteral {
    fn into_output_actions<'cx>(&self, ecx: &'cx ExtCtxt) -> Vec<OutputAction> {
        match self.val {
            LitValue::LitString(ref contents) => {
                vec![OutputAction::Write(contents.to_owned())]
            }
        }
    }
}

impl IntoOutputActions for TemplateNode {
    fn into_output_actions<'cx>(&self, ecx: &'cx ExtCtxt) -> Vec<OutputAction> {
        match self {
            &TemplateNode::ElementNode(ref element) => element.into_output_actions(ecx),
            &TemplateNode::ExprNode(ref simple_expr) => simple_expr.into_output_actions(ecx),
            &TemplateNode::LiteralNode(ref template_literal) => template_literal.into_output_actions(ecx),
        }
    }
}

pub fn parse_element<'cx, 'a>(ecx: &'cx ExtCtxt,
                              mut parser: &mut Parser<'a>,
                              span: Span,
                              element_type: String)
                              -> PResult<'a, Element> {
    try!(parser.expect(&token::OpenDelim(token::Bracket)));

    let nodes = try!(parse_contents(ecx, &mut parser, span));

    Ok(Element {
        element_type: element_type,
        span: span,
        nodes: nodes,
    })
}

fn parse_expr_node<'cx, 'a>(ecx: &'cx ExtCtxt, mut parser: &mut Parser<'a>, span: Span) -> PResult<'a, TemplateNode> {
    let simple_expr = try!(parse_simple_expr(ecx, &mut parser, span));
    Ok(TemplateNode::ExprNode(simple_expr))
}

/*
pub fn parse_expr_or_lit<'cx, 'a>(ecx: &'cx ExtCtxt,
                           mut parser: &mut Parser<'a>,
                           span: Span)
                           -> PResult<'a, TemplateNode> {
    match &parser.token {
        &token::OpenDelim(token::Paren) => {
            let expr = try!(parser.parse_expr());
            parser.bump();

            let tokens: Vec<SimpleExprToken> = vec![];
            

            let tokens = expr.to_simple_expr_tokens();
            return Ok(TemplateNode::ExprNode(TemplateExpr { span: span, tokens: tokens }));
        },

        // Otherwise, assume we have a literal expr
        _ => {
            //let expr = try!(parser.parse_expr());
            let (str_, _) = try!(parser.parse_str());
            parser.bump();

            let s = String::from(str_.to_string());
            return Ok(TemplateNode::LiteralNode(TemplateLiteral { span: span, val: LitValue::LitString(s) }));
        }
    }
}
*/

pub fn parse_node<'cx, 'a>(ecx: &'cx ExtCtxt,
                           mut parser: &mut Parser<'a>,
                           span: Span)
                           -> PResult<'a, TemplateNode> {
    let is_open = parser.token == token::OpenDelim(token::Brace);

    if is_open {
        // Parse simple expression node
        let node = try!(parse_expr_node(ecx, &mut parser, span));

        Ok(node)
    } else {
        ecx.span_warn(span, "Expecting nested element");

        // Nested element
        let element_type = try!(parser.parse_ident());
        let element =
            try!(parse_element(ecx, &mut parser, span, element_type.name.to_string()));

        Ok(TemplateNode::ElementNode(element))
    }
}

pub fn parse_contents<'cx, 'a>(ecx: &'cx ExtCtxt,
                               mut parser: &mut Parser<'a>,
                               span: Span)
                               -> PResult<'a, Vec<TemplateNode>> {
    let mut nodes: Vec<TemplateNode> = Vec::new();

    loop {
        match parser.token {
            token::OpenDelim(token::Bracket) => {
                // Start of expression, which can be a literal value
                ecx.span_warn(span, "Got open expression");
                let node = try!(parse_expr_node(ecx, &mut parser, span));
                nodes.push(node);
                break;
            },

            token::CloseDelim(token::Bracket) => {
                ecx.span_warn(span, "Got close contents");
                break;
            }

            _ => {
                ecx.span_warn(span, "Parsing node");
                let node = try!(parse_node(ecx, parser, DUMMY_SP));
                nodes.push(node);
            }
        }
    }

    Ok(nodes)
}
