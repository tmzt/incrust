#![crate_name="incrust_plugin"]
#![crate_type="dylib"]
#![feature(quote, plugin_registrar, rustc_private)]

#[macro_use]
extern crate log;

extern crate syntax;
extern crate rustc;
extern crate rustc_plugin;

use rustc_plugin::Registry;

use std::fmt::Write;

use syntax::abi::Abi;
use syntax::ast::{self, DUMMY_NODE_ID};

use syntax::codemap::{Span, Spanned, dummy_spanned, respan, spanned, DUMMY_SP};
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult};
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::ToTokens;
use syntax::print::pprust::{token_to_string, tts_to_string};
use syntax::tokenstream::TokenTree;
use syntax::util::small_vector::SmallVector;
use syntax::parse::{token, PResult};
use syntax::parse::common::SeqSep;
use syntax::parse::parser::Parser;
use syntax::ptr::P;


#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("emit_rust_template", emit_rust_template);
}

trait IntoViewItem {
    fn into_view_item<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Item>;
}

trait IntoBlock {
    fn into_block<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Block>;
}

trait IntoWriteStmt {
    fn into_write_stmt<'cx>(&self, ecx: &'cx ExtCtxt, w: ast::Ident) -> ast::Stmt;
}

trait IntoOutputActions {
    fn into_output_actions<'cx>(&self, ecx: &'cx ExtCtxt) -> Vec<OutputAction>;
}

struct View {
    name: String,
    span: Span,
    nodes: Vec<TemplateNode>
}

impl IntoViewItem for View {
    fn into_view_item<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Item> {
        create_view_item(ecx, self.span, &self)
    }
}

struct Element {
    element_type: String,
    span: Span,
    nodes: Vec<TemplateNode>
}

struct TemplateExpr {
    span: Span,
    tokens: Vec<TokenTree>
}

// Represents a parsed node in the template syntax
enum TemplateNode {
    ElementNode(Element),
    ExprNode(TemplateExpr)
}

// Represents a type of action to perform when rendering
enum OutputAction {
    // Text and computed values
    Write(String),
    WriteResult(TemplateExpr),

    // Elements
    WriteOpen(String),
    WriteClose(String),
    WriteVoid(String)

    //CallTemplate(String),
    //WriteElement(Element),
}

impl IntoWriteStmt for OutputAction {
    fn into_write_stmt<'cx>(&self, ecx: &'cx ExtCtxt, w: ast::Ident) -> ast::Stmt {
        match *self {
            OutputAction::Write(ref contents) => {
                //let w_name =  w.name.to_string();
                let stmt = quote_stmt!(ecx,
                    {
                        println!("Writing contents [{}] to ${}", $contents, "out");
                        write!($w, "{}", $contents);
                    }
                ).unwrap();

                stmt
            },

            // For now, write the expression as a string
            OutputAction::WriteResult(ref template_expr) => {
                let contents = tts_to_string(&template_expr.tokens);
                let stmt = quote_stmt!(ecx,
                    {
                        println!("Writing contents [{}] to ${}", $contents, "out");
                        write!(out, $contents);
                    }
                ).unwrap();

                stmt
            },

            OutputAction::WriteOpen(ref element_type) => {
                let contents = format!("<{}>", element_type);
                let stmt = quote_stmt!(ecx,
                    {
                        println!("Writing contents [{}] to ${}", $contents, "out");
                        write!(out, $contents);
                    }
                ).unwrap();

                stmt
            },

            OutputAction::WriteClose(ref element_type) => {
                let contents = format!("</{}>", element_type);
                let stmt = quote_stmt!(ecx,
                    {
                        println!("Writing contents [{}] to ${}", $contents, "out");
                        write!(out, $contents);
                    }
                ).unwrap();

                stmt
            },

            OutputAction::WriteVoid(ref element_type) => {
                let contents = format!("<{} />", element_type);
                let stmt = quote_stmt!(ecx,
                    {
                        println!("Writing contents [{}] to ${}", $contents, "out");
                        write!(out, $contents);
                    }
                ).unwrap();

                stmt
            }            
        }
    }
}

impl IntoOutputActions for Element {
    fn into_output_actions<'cx>(&self, ecx: &'cx ExtCtxt) -> Vec<OutputAction> {
        let nodes = &self.nodes;
        let element_type = &self.element_type;
        let mut output_actions = Vec::new();

        output_actions.push(OutputAction::WriteOpen(element_type.clone()));

        let child_actions : Vec<OutputAction> = nodes.iter()
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

impl IntoOutputActions for TemplateNode {
    fn into_output_actions<'cx>(&self, ecx: &'cx ExtCtxt) -> Vec<OutputAction> {
        match self {
            &TemplateNode::ElementNode(ref element) => element.into_output_actions(ecx),
            &TemplateNode::ExprNode(ref template_expr) => template_expr.into_output_actions(ecx)
        }
    }
}

impl IntoBlock for View {
    fn into_block<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Block> {
        let name = &self.name;
        let nodes = &self.nodes;

        let w_ident = ecx.ident_of("out");
        let mut stmts = Vec::new();

        let out_stmt = quote_stmt!(ecx, let mut $w_ident = String::new()).unwrap();
        stmts.push(out_stmt);

        let write_stmts: Vec<ast::Stmt> = nodes.iter()
            .flat_map(|node| node.into_output_actions(ecx))
            .map(|output_action| output_action.into_write_stmt(ecx, w_ident))
            .collect();
        stmts.extend(write_stmts);

        // Return rendered string for now
        stmts.push(quote_stmt!(ecx, $w_ident).unwrap());

        ecx.block(self.span, stmts)
    }
}

fn parse_element<'cx, 'a>(ecx: &'cx ExtCtxt, mut parser: &mut Parser<'a>, span: Span, element_type: String) -> PResult<'a, Element> {
    try!(parser.expect(&token::OpenDelim(token::Bracket)));

    let nodes = try!(parse_contents(ecx, &mut parser, span));

    Ok(Element { element_type: element_type, span: span, nodes: nodes })
}

fn parse_template_expr<'a>(parser: &mut Parser<'a>, span: Span) -> PResult<'a, TemplateExpr> {
    try!(parser.expect(&token::OpenDelim(token::Brace)));

    let tokens = parser.parse_seq_to_end(
        &token::CloseDelim(token::Brace),
        SeqSep::none(),
        |parser| parser.parse_token_tree())
        .unwrap();

    Ok(TemplateExpr { span: span, tokens: tokens })
}

fn parse_node<'cx, 'a>(ecx: &'cx ExtCtxt, mut parser: &mut Parser<'a>, span: Span) -> PResult<'a, TemplateNode> {
    let is_open = parser.token == token::OpenDelim(token::Brace);

    if is_open {
        // Template expression
        let template_expr = try!(parse_template_expr(&mut parser, DUMMY_SP));
        Ok(TemplateNode::ExprNode(template_expr))
    } else {
        ecx.span_warn(span, "Expecting nested element");

        // Nested element
        let element_type = try!(parser.parse_ident());
        let element = try!(parse_element(ecx, &mut parser, DUMMY_SP, element_type.name.to_string()));

        Ok(TemplateNode::ElementNode(element))
    }
}

fn parse_contents<'cx, 'a>(ecx: &'cx ExtCtxt, parser: &mut Parser<'a>, span: Span) -> PResult<'a, Vec<TemplateNode>> {
    let mut nodes : Vec<TemplateNode> = Vec::new();

    loop {
        match parser.token {
            token::CloseDelim(token::Bracket) => {
                ecx.span_warn(span, "Got close contents");
                break;
            },

            _ => {
                ecx.span_warn(span, "Parsing node");
                let node = try!(parse_node(ecx, parser, DUMMY_SP));
                nodes.push(node);
            }
        }
    }

    Ok(nodes)
}

fn parse_view<'cx, 'a>(ecx: &'cx ExtCtxt, parser: &mut Parser<'a>, span: Span) -> PResult<'a, View> {
    let view_token = parser.parse_ident().unwrap();
    let view_name = parser.parse_ident().unwrap();

    try!(parser.expect(&token::OpenDelim(token::Bracket)));

    let nodes = try!(parse_contents(ecx, parser, span));

    Ok(View { name: view_name.name.to_string(), span: span, nodes: nodes })
}

fn create_view_item<'cx>(ecx: &'cx ExtCtxt, span: Span, view: &View) -> P<ast::Item> {
    let name = ecx.ident_of(&format!("rusttemplate_view_{}", view.name));
    let block = view.into_block(ecx);

    let inputs = vec![];
    let ret_ty = quote_ty!(ecx, String);
    ecx.item_fn(DUMMY_SP, name, inputs, ret_ty, block)
}

fn create_template_block<'cx>(ecx: &'cx ExtCtxt, span: Span, views: Vec<View>) -> Box<MacResult + 'cx> {
    let view_item_stmts: Vec<ast::Stmt> = views.iter()
        .map(|view| view.into_view_item(ecx))
        .map(|item| ecx.stmt_item(span, item))
    .collect();

    let mut stmts = Vec::new();
    stmts.extend(view_item_stmts);

    let name = ecx.ident_of("rusttemplate_view_root");
    let args = vec![];
    let call_expr = ecx.expr_call_ident(span, name, args);
    stmts.push(ecx.stmt_expr(call_expr));

    let block = ecx.block(span, stmts);

    MacEager::expr(ecx.expr_block(block))
}

fn parse_template<'cx, 'a>(ecx: &'cx ExtCtxt, parser: &mut Parser<'a>) -> Box<MacResult + 'cx> {
    match parse_view(ecx, parser, DUMMY_SP) {
        Ok(view) => {
            let views = vec![view];
            create_template_block(ecx, DUMMY_SP, views)
        },

        Err(mut err) => {
            err.emit();
            DummyResult::expr(DUMMY_SP)
        }
    }
}

fn emit_rust_template<'cx>(
        ecx: &'cx mut ExtCtxt,
        span: Span,
        tts: &[TokenTree]) -> Box<MacResult + 'cx> {

    let mut parser = ecx.new_parser_from_tts(tts);
    parse_template(ecx, &mut parser)
}