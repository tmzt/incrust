
use syntax::ast;
use syntax::ptr::P;
use syntax::codemap::{DUMMY_SP, Span, Spanned, respan};
use syntax::ext::base::ExtCtxt;
use syntax::ext::quote::rt::ToTokens;
use syntax::tokenstream::TokenTree;
use syntax::parse::{PResult, token};
use syntax::parse::parser::Parser;

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

/*
impl ToTokens for P<Node> {
    fn to_tokens(&self, ecx: &ExtCtxt) -> Vec<TokenTree> {
        vec![TokenTree::Token(self.span, token::Interpolated(token::NtItem(self.clone())))]
    }
}
*/

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

pub fn parse_action_expr<'cx, 'a>(ecx: &'cx mut ExtCtxt, store_name: &str, span: Span, parser: &mut Parser<'a>) -> PResult<'a, ActionExpr> {
    let mut toks: Vec<P<Node>> = vec![];
    let ctx = ExprCtx::new();
    loop {
        ecx.span_warn(span, &format!("Got token: {:?}", parser.token));
        match &parser.token {
            &token::Ident(ident) => {
                let ident_name = ident.name.to_string();
                match ident_name {
                    _ if ident_name == store_name => {
                        toks.push(ctx.store_ref(span, store_name));
                    },
                    _ => {
                        ecx.span_err(span, &format!("Invalid variable reference ({}) in expression, must match store name.", ident_name));
                    }
                };
                parser.bump();
        },
        &token::OpenDelim(token::Paren) => {
            parser.eat(&token::OpenDelim(token::Paren));
        },
        &token::CloseDelim(token::Paren) => {
            parser.eat(&token::CloseDelim(token::Paren));
        },
        &token::Eof => { break; },

        _ => {
            ecx.span_warn(span, &format!("Unexpected token parsing action expression: {:?}", parser.token));

            // Ignore other tokens for now
            parser.bump();
        }
        //_ => { ecx.span_err(span, "Unexpected token in expression"); }
        };
    }
    Ok(ActionExpr{store_name: String::from(store_name), span: span, nodes: toks })
}
