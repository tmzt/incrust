
use syntax::ast;
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult};
use syntax::ext::build::AstBuilder;
use syntax::codemap::{Span, Spanned, dummy_spanned, respan, spanned, DUMMY_SP};
use syntax::parse::{token, PResult};
use syntax::parse::parser::Parser;
use syntax::ptr::P;

use node::{Element, TemplateExpr, TemplateNode, parse_node, parse_contents};
use codegen::{IntoWriteStmt, IntoViewItem};
use jsgen::{IntoJsFunction, IntoJsOutputCall};
use output_actions::{OutputAction, IntoOutputActions};
use IntoBlock;



pub struct View {
    name: String,
    span: Span,
    nodes: Vec<TemplateNode>
}

impl View {
    pub fn name(&self) -> &str {
        &self.name
    }
}

fn create_view_item<'cx>(ecx: &'cx ExtCtxt, span: Span, view: &View) -> P<ast::Item> {
    let name = ecx.ident_of(&format!("rusttemplate_view_{}", view.name));
    let block = view.into_block(ecx);

    let inputs = vec![];
    let ret_ty = quote_ty!(ecx, String);
    ecx.item_fn(DUMMY_SP, name, inputs, ret_ty, block)
}

impl IntoViewItem for View {
    fn into_view_item<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Item> {
        create_view_item(ecx, self.span, &self)
    }
}

impl IntoOutputActions for View {
    fn into_output_actions<'cx>(&self, ecx: &'cx ExtCtxt) -> Vec<OutputAction> {
        let name = &self.name;
        let nodes = &self.nodes;

        let w_ident = ecx.ident_of("out");
        let mut stmts = Vec::new();

        let out_stmt = quote_stmt!(ecx, let mut $w_ident = String::new()).unwrap();
        stmts.push(out_stmt);

        let output_actions: Vec<OutputAction> = nodes.iter()
            .flat_map(|node| node.into_output_actions(ecx))
            .collect();

        output_actions
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

impl IntoJsFunction for View {
    fn into_js_function<'cx>(&self, ecx: &'cx ExtCtxt) -> String {
        let name = &self.name;
        let output_actions = &self.into_output_actions(ecx);

        let js_stmts: Vec<String> = output_actions.iter()
            .map(|output_action| output_action.into_js_output_call())
            .collect();
        
        let js_body: String = js_stmts.join(" ");
        let js = format!("function render_view_{}(data) {{ {} }}", name, js_body);

        js
    }
}

pub fn parse_view<'cx, 'a>(ecx: &'cx ExtCtxt,
                       parser: &mut Parser<'a>,
                       span: Span)
                       -> PResult<'a, View> {
    let view_token = parser.parse_ident().unwrap();
    let view_name = parser.parse_ident().unwrap();

    try!(parser.expect(&token::OpenDelim(token::Bracket)));

    let nodes = try!(parse_contents(ecx, parser, span));

    Ok(View {
        name: view_name.name.to_string(),
        span: span,
        nodes: nodes,
    })
}
