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

trait IntoOutputAction {
    fn into_output_action<'cx>(&self, ecx: &'cx ExtCtxt) -> OutputAction;
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
    tokens: Vec<TokenTree>
}

struct TemplateExpr {
    span: Span,
    tokens: Vec<TokenTree>
}

struct TextNode {
    contents: String,
    span: Span
}

// Represents a parsed node in the template syntax
enum TemplateNode {
    ElementNode(Element),
    ExprNode(TemplateExpr)
}

// Represents a type of action to perform when rendering
enum OutputAction {
    Write(String),
    WriteResult(TemplateExpr)
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
            }
        }
    }
}

/*
impl IntoBlock for Iterator<Item=IntoWriteStmt> {
    fn into_block<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Block> {
        let w = quote_ty!(output);
        let stmts = self.map(|ws| ws.into_write_stmt(ecx, None)).collect();

        ecx.block(DUMMY_SP, stmts)
    }
}
*/

impl IntoOutputAction for Element {
    fn into_output_action<'cx>(&self, ecx: &'cx ExtCtxt) -> OutputAction {
        // For now, output the element as a token string
        let contents = tts_to_string(&self.tokens);
        OutputAction::Write(contents)
    }
}

impl IntoOutputAction for TemplateExpr {
    fn into_output_action<'cx>(&self, ecx: &'cx ExtCtxt) -> OutputAction {
        // For now, output the element as a token string
        let contents = tts_to_string(&self.tokens);
        OutputAction::Write(contents)
    }
}

impl IntoOutputAction for TemplateNode {
    fn into_output_action<'cx>(&self, ecx: &'cx ExtCtxt) -> OutputAction {
        match self {
            &TemplateNode::ElementNode(ref element) => element.into_output_action(ecx),
            &TemplateNode::ExprNode(ref template_expr) => template_expr.into_output_action(ecx)
        }
    }
}

impl IntoBlock for Element {
    fn into_block<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Block> {
        let element_type = &self.element_type;
        let stmt = quote_stmt!(ecx,
            {
                println!("Opening and closing [{}] element", $element_type);
                format!("<{}></{}>", $element_type, $element_type)
            }
        ).unwrap();

        let stmts = vec![ stmt ];
        ecx.block(self.span, stmts)
    }
}

impl IntoBlock for View {
    fn into_block<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Block> {
        let name = &self.name;
        let nodes = &self.nodes;

        let w_ident = ecx.ident_of("out");
        let mut stmts = Vec::new();

//        let string_expr = quote_stmt!(ecx, String::new());
//       //let out_stmt = ecx.stmt_let(DUMMY_SP, true, w_ident, ecx.expr_vec_ng(DUMMY_SP));
//        let out_stmt = ecx.stmt_let(DUMMY_SP, true, w_ident, string_expr);
        let out_stmt = quote_stmt!(ecx, let mut $w_ident = String::new()).unwrap();
        stmts.push(out_stmt);

        let write_stmts: Vec<ast::Stmt> = nodes.iter()
            .map(|node| node.into_output_action(ecx))
            .map(|output_action| output_action.into_write_stmt(ecx, w_ident))
            .collect();
        stmts.extend(write_stmts);

//        stmts.push(quote_stmt!(ecx, println!(format!("$out contains [{}]", $w_ident))).unwrap());
        stmts.push(quote_stmt!(ecx, $w_ident).unwrap());

        ecx.block(self.span, stmts)
    }
}

/*
impl IntoBlock for TextNodes {
    fn into_block<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Block> {
        let contents = &self.contents;
        let stmt = quote_stmt!(ecx,
            {
                println!("Output text nodes as block");
                format!("{}", $contents)
            }
        ).unwrap();

        let stmts = vec![ stmt ];
        ecx.block(self.span, stmts)
    }
}
*/

/*
impl IntoBlock for TemplateNode {
    fn into_block<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Block> {
        match *self {
            TemplateNode::ElementNode(ref element) => { element.into_block() },
            TemplateNode::TextNodes(tokens) => {

                let s = tts_to_string(tokens);
                let text_node = TextNode { contents: s, span: DUMMY_SP };

                text_node.into_block(ecx)
            }
        }
    }
}
*/

fn parse_element<'a>(parser: &mut Parser<'a>, span: Span, element_type: String) -> PResult<'a, Element> {
    try!(parser.expect(&token::OpenDelim(token::Bracket)));

    let tokens = parser.parse_seq_to_end(
        &token::CloseDelim(token::Bracket),
        SeqSep::none(),
        |parser| parser.parse_token_tree())
        //|parser| parse_contents(parser, DUMMY_SP))
        .unwrap();

    Ok(Element { element_type: element_type, span: span, tokens: tokens })
}

fn parse_template_expr<'a>(parser: &mut Parser<'a>, span: Span) -> PResult<'a, TemplateExpr> {
    try!(parser.expect(&token::OpenDelim(token::Bracket)));

    let tokens = parser.parse_seq_to_end(
        &token::CloseDelim(token::Bracket),
        SeqSep::none(),
        |parser| parser.parse_token_tree())
        .unwrap();

    Ok(TemplateExpr { span: span, tokens: tokens })
}

fn parse_node<'cx, 'a>(ecx: &'cx ExtCtxt, mut parser: &mut Parser<'a>, span: Span) -> PResult<'a, TemplateNode> {
    let is_open = { parser.look_ahead(1, |t| *t == token::OpenDelim(token::Brace)) };
    if is_open {
        parser.eat(&token::OpenDelim(token::Brace));

        // Template expression
        let template_expr = parse_template_expr(&mut parser, DUMMY_SP).unwrap();
        Ok(TemplateNode::ExprNode(template_expr))
    } else {
        ecx.span_warn(span, "Expecting nested element");

        // Nested element
        let element_type = try!(parser.parse_ident());
        let element = try!(parse_element(&mut parser, DUMMY_SP, element_type.name.to_string()));

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

    /*
    while !parser.look_ahead(1, |t| *t == token::CloseDelim(token::Bracket)) {
        ecx.span_warn(span, "Parsing node");
        let node = try!(parse_node(ecx, parser, DUMMY_SP));
        nodes.push(node);
    }
    */

    Ok(nodes)
}

fn parse_view<'cx, 'a>(ecx: &'cx ExtCtxt, parser: &mut Parser<'a>, span: Span) -> PResult<'a, View> {
    let view_token = parser.parse_ident().unwrap();
    let view_name = parser.parse_ident().unwrap();

    try!(parser.expect(&token::OpenDelim(token::Bracket)));

    /*
    let tokens = parser.parse_seq_to_end(
        &token::CloseDelim(token::Bracket),
        SeqSep::none(),
        |pp| pp.parse_token_tree())
        .unwrap();
    */

    let nodes = try!(parse_contents(ecx, parser, span));
    //let nodes = nodes.iter().flat_map(|node| node).collect();

    Ok(View { name: view_name.name.to_string(), span: span, nodes: nodes })
}

fn create_view_item<'cx>(ecx: &'cx ExtCtxt, span: Span, view: &View) -> P<ast::Item> {
    let name = ecx.ident_of(&format!("rusttemplate_view_{}", view.name));
    let block = view.into_block(ecx);

    /*
    let mut parser = ecx.new_parser_from_tts(&view.tokens);
    let element = parse_element(&mut parser, span).unwrap();
    let block = element.into_block(ecx);
    */

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

    /*
    let block = ecx.block(span, vec![
            ecx.stmt_item(span, item),
            ecx.stmt_expr(call_expr)]);
    */
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

    /*
    let mut i = 0;
    loop {
        match (tts.get(i), tts.get(i+1), tts.get(i+2)) {
            (Some(&TokenTree::Token(_, token::Ident(element_type))), _, _) => {
                ecx.span_warn(span, &format!("Outputing elementOpen for {}", &element_type.to_string()));

                let mut parser = ecx.new_parser_from_tts(tts);
                return parse_template(ecx, &mut parser);
            },
            (Some(_), _, _) => break,
            (None, _, _) => break
        }
    }
    */

    //MacEager::stmts(SmallVector::many(result))
    //MacEager::items(SmallVector::many(items))
    //DummyResult::any(span)
}