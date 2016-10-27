
use syntax::ast;
use syntax::ptr::P;
use syntax::codemap::{DUMMY_SP, Span, Spanned, respan};
use syntax::ext::base::ExtCtxt;
use syntax::ext::quote::rt::ToTokens;
use syntax::tokenstream::TokenTree;
use syntax::parse::{PResult, token};
use syntax::parse::token::BinOpToken as binops;
use syntax::parse::parser::Parser;
use syntax::parse::common::SeqSep;

use std::ops::Deref;


#[derive(Debug)]
pub enum BinOpType {
    Add,
    Subtract,
    Multiply,
    Divide
}

#[derive(Debug)]
pub struct LiteralData {
    val: Vec<TokenTree>
}

#[derive(Debug)]
pub enum Literal {
    Str(LiteralData),
    Number(LiteralData)
}

#[derive(Debug)]
pub enum NodeKind {
//    Expr(String),
    BinOp(BinOpType, P<Node>, P<Node>),
    Lit(Literal),
    StoreRef(String)
}
pub type Node = Spanned<NodeKind>;

impl ToTokens for NodeKind {
    fn to_tokens(&self, ecx: &ExtCtxt) -> Vec<TokenTree> {
        ecx.span_warn(DUMMY_SP, "NodeKind ToTokens");
        let mut res = vec![];
        match self {
            &NodeKind::StoreRef(ref store_name) => {
                res.push(TokenTree::Token(DUMMY_SP, token::Ident(ecx.ident_of(store_name))));
            },

            _ => {
                // Ignore for now
                ecx.span_warn(DUMMY_SP, "Not a store ref in ToTokens");
            }
        };
        res
    }
}

fn nodes_to_tts(ecx: &ExtCtxt, span: Span, nodes: &[P<Node>]) -> Vec<TokenTree> {
    ecx.span_warn(span, "nodes_to_tts");

    ecx.span_warn(span, &format!("nodes: {:?}", nodes));
    let res = nodes.iter()
        .flat_map(|p| p.node.to_tokens(ecx)).collect();
    ecx.span_warn(span, &format!("tokens: {:?}", res));
    res
}
pub fn action_expr_to_tts(ecx: &ExtCtxt, expr: ActionExpr) -> Vec<TokenTree> {
    nodes_to_tts(ecx, expr.span, &expr.nodes)
}

pub struct ActionExpr {
    store_name: String,
    span: Span,
    nodes: Vec<P<Node>>
}

struct ExprCtx {

}
impl ExprCtx {
    pub fn new() -> ExprCtx { ExprCtx { } }

    pub fn binop(&self, span: Span, op: BinOpType, left: P<Node>, right: P<Node>) -> P<Node> {
        P(respan(span, NodeKind::BinOp(op, left, right)))
    }

    pub fn store_ref(&self, span: Span, store_name: &str) -> P<Node> {
        P(respan(span, NodeKind::StoreRef(String::from(store_name))))
    }
}

pub fn parse_limited_expr_block<'cx, 'a>(ecx: &'cx mut ExtCtxt, span: Span, store_name: &str, parser: &mut Parser<'a>) -> PResult<'a, Vec<TokenTree>> {
    try!(parser.expect(&token::OpenDelim(token::Paren)));

    ecx.span_warn(span, &format!("Store: {}", store_name));
    let expr_tokens = parser.parse_seq_to_end(&token::CloseDelim(token::Paren),
                        SeqSep::none(),
                        |parser| {
                            ecx.span_warn(span, &format!("Got token: {:?}", &parser.token));

                            match &parser.token {
                                &token::BinOp(op) => {
                                    match op {
                                        binops::Plus | binops::Minus | binops::Star | binops::Slash => {
                                        },
                                        _ => {
                                            ecx.span_err(span, &format!("Unexpected binop: {:?}", op));
                                        }
                                    };
                                },
                                &token::Ident(ident) => {
                                    let ident_name = ident.name.to_string();
                                    if ident_name != store_name {
                                        ecx.span_err(span, &format!("Invalid variable reference ({}) in expression, must match the store name ({}).", ident_name, store_name));
                                    }
                                },

                                &token::OpenDelim(token::Brace) => {},
                                &token::CloseDelim(token::Brace) => {},
                                &token::Colon => {},

                                &token::Interpolated(token::NtExpr(_)) => {},
                                &token::Literal(_, _) => {},
                                &token::Eof => {},
                                _ => {
                                            ecx.span_err(span, &format!("Unexpected token: {:?}", &parser.token));
                                }
                            };
                            parser.parse_token_tree()
                        }).unwrap();
    Ok(expr_tokens)
}