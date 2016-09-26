
use syntax::ast;
use syntax::tokenstream::TokenTree;
use syntax::codemap::{Span, Spanned, dummy_spanned, respan, spanned, DUMMY_SP};
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult};
use syntax::print::pprust::{token_to_string, tts_to_string};
use syntax::parse::{token, PResult};
use syntax::parse::common::SeqSep;
use syntax::parse::parser::Parser;
use syntax::ptr::P;

use codegen::IntoWriteStmt;
use output_actions::{OutputAction, IntoOutputActions};


pub struct Element {
    element_type: String,
    span: Span,
    nodes: Vec<TemplateNode>,
}

pub struct TemplateExpr {
    span: Span,
    tokens: Vec<TokenTree>,
}

// Represents a parsed node in the template syntax
pub enum TemplateNode {
    ElementNode(Element),
    ExprNode(TemplateExpr),
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

impl IntoOutputActions for TemplateExpr {
    fn into_output_actions<'cx>(&self, ecx: &'cx ExtCtxt) -> Vec<OutputAction> {
        // For now, output the element as a token string
        let contents = tts_to_string(&self.tokens);
        vec![OutputAction::Write(contents)]
    }
}

// Used to output expression as a token string
impl IntoWriteStmt for TemplateExpr {
    fn into_write_stmt<'cx>(&self, ecx: &'cx ExtCtxt, w: ast::Ident) -> ast::Stmt {
        let contents = tts_to_string(&self.tokens);
        let stmt = quote_stmt!(ecx, {
                println!("Writing contents [{}] to ${}", $contents, "out");
                write!(out, $contents);
            }).unwrap();

        stmt
    }
}

impl IntoOutputActions for TemplateNode {
    fn into_output_actions<'cx>(&self, ecx: &'cx ExtCtxt) -> Vec<OutputAction> {
        match self {
            &TemplateNode::ElementNode(ref element) => element.into_output_actions(ecx),
            &TemplateNode::ExprNode(ref template_expr) => template_expr.into_output_actions(ecx),
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

pub fn parse_template_expr<'a>(parser: &mut Parser<'a>, span: Span) -> PResult<'a, TemplateExpr> {
    try!(parser.expect(&token::OpenDelim(token::Brace)));

    let tokens = parser.parse_seq_to_end(&token::CloseDelim(token::Brace),
                          SeqSep::none(),
                          |parser| parser.parse_token_tree())
        .unwrap();

    Ok(TemplateExpr {
        span: span,
        tokens: tokens,
    })
}

pub fn parse_node<'cx, 'a>(ecx: &'cx ExtCtxt,
                           mut parser: &mut Parser<'a>,
                           span: Span)
                           -> PResult<'a, TemplateNode> {
    let is_open = parser.token == token::OpenDelim(token::Brace);

    if is_open {
        // Template expression
        let template_expr = try!(parse_template_expr(&mut parser, DUMMY_SP));
        Ok(TemplateNode::ExprNode(template_expr))
    } else {
        ecx.span_warn(span, "Expecting nested element");

        // Nested element
        let element_type = try!(parser.parse_ident());
        let element =
            try!(parse_element(ecx, &mut parser, DUMMY_SP, element_type.name.to_string()));

        Ok(TemplateNode::ElementNode(element))
    }
}

pub fn parse_contents<'cx, 'a>(ecx: &'cx ExtCtxt,
                               parser: &mut Parser<'a>,
                               span: Span)
                               -> PResult<'a, Vec<TemplateNode>> {
    let mut nodes: Vec<TemplateNode> = Vec::new();

    loop {
        match parser.token {
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
