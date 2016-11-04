
use syntax::tokenstream::TokenTree;
use syntax::codemap::{Span, DUMMY_SP};
use syntax::ext::base::ExtCtxt;
use syntax::ext::quote::rt::ToTokens;
use syntax::parse::{token, PResult};
use syntax::parse::parser::Parser;

use output_actions::{OutputAction, IntoOutputActions};
use simple_expr::SimpleExpr;
use simple_expr::parse::parse_simple_expr;

use nodes::content_node::ContentNode;

/// Represents a parsed view in template contents
#[derive(Clone, Debug)]
pub struct View {
    name: String,
    span: Span,
    nodes: Vec<ContentNode>
}

impl View {
    pub fn name(&self) -> &str {
        &self.name
    }
}

/*
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
*/

pub mod parse {
    use super::View;
    use syntax::tokenstream::TokenTree;
    use syntax::codemap::{Span, DUMMY_SP};
    use syntax::ext::base::ExtCtxt;
    use syntax::parse::{token, PResult};
    use syntax::parse::parser::Parser;
    use nodes::content_node::parse::{NodeType, parse_contents};

    pub fn parse_view<'cx, 'a>(ecx: &'cx ExtCtxt, mut parser: &mut Parser<'a>, span: Span) -> PResult<'a, View> {
        //let view_token = try!(parser.parse_ident());
        let view_name = try!(parser.parse_ident());

        try!(parser.expect(&token::OpenDelim(token::Bracket)));

        let nodes = try!(parse_contents(ecx, parser, span, &NodeType::Root));

        Ok(View {
            name: view_name.name.to_string(),
            span: span,
            nodes: nodes,
        })
    }
}

mod output {
    use super::View;
    use syntax::ext::base::ExtCtxt;
    use output_actions::{OutputAction, IntoOutputActions, WriteOutputActions, OutputActionWrite};

    impl IntoOutputActions for View {
        fn into_output_actions<'cx>(&self) -> Vec<OutputAction> {
            let name = &self.name;
            let nodes = &self.nodes;

            let output_actions: Vec<OutputAction> = nodes.iter()
                .flat_map(|node| node.into_output_actions())
                .collect();

            output_actions
        }
    }

    impl WriteOutputActions for View {
        fn write_output_actions(&self, w: &mut OutputActionWrite) {
            for node in &self.nodes {
                node.write_output_actions(w);
            }
        }
    }
}

mod output_ast {
    use super::View;
    use syntax::ast;
    use syntax::codemap::{Span, DUMMY_SP};
    use syntax::ext::base::ExtCtxt;
    use syntax::ext::build::AstBuilder;
    use syntax::ptr::P;
    use output_actions::{OutputAction};
    use codegen::IntoBlock;
    use codegen::lang::{Lang, Html, Js};
    use codegen::item_writer::{WriteItems, ItemWrite};
    use codegen::output_string_writer::{WriteOutputStrings, OutputStringWrite};
    use codegen::output_stmt_writer::{WriteOutputStmts, OutputStmtWrite};

    impl IntoBlock for View {
        fn into_block<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Block> {
            let mut stmts = vec![];
            let w_ident = ecx.ident_of("out");
            &self.write_output_stmts(ecx, &mut stmts, w_ident);
            stmts.push(quote_stmt!(ecx, let mut $w_ident = String::new()).unwrap());
            ecx.block(DUMMY_SP, stmts)            
        }
    }

    fn create_view_item<'cx>(ecx: &'cx ExtCtxt, span: Span, view: &View) -> P<ast::Item> {
        let name = ecx.ident_of(&format!("rusttemplate_view_{}", view.name()));
        let block = view.into_block(ecx);

        let inputs = vec![];
        let ret_ty = quote_ty!(ecx, String);
        ecx.item_fn(span, name, inputs, ret_ty, block)
    }

    impl WriteItems for View {
        fn write_items<'s, 'cx>(&self, ecx: &'cx ExtCtxt, w: &'s mut ItemWrite) {
            let name = ecx.ident_of(&format!("rusttemplate_view_{}", &self.name()));
            let block = self.into_block(ecx);

            let inputs = vec![];
            let ret_ty = quote_ty!(ecx, String);
            let item = ecx.item_fn(DUMMY_SP, name, inputs, ret_ty, block);
            w.write_item(ecx, item);
        }
    }

    /*
    pub fn create_view_item<'cx>(ecx: &'cx ExtCtxt, view: &View) -> P<ast::Item> {
        let mut stmts: Vec<ast::Stmt> = vec![];
        &view.write_stmts(ecx, &mut stmts);

        let w_ident = ecx.ident_of("out");
        stmts.push(quote_stmt!(ecx, let mut $w_ident = String::new()).unwrap());
        let block = ecx.block(DUMMY_SP, stmts);

        let name = ecx.ident_of(&format!("rusttemplate_view_{}", view.name()));
        let inputs = vec![];
        let ret_ty = quote_ty!(ecx, String);
        ecx.item_fn(DUMMY_SP, name, inputs, ret_ty, block)
    }
    */

    /*
    impl WriteItems for View {
        fn write_items<'cx>(&self, ecx: &'cx mut ExtCtxt, w: &mut ItemWrite) {
            let item = create_view_item(ecx, &self);
            ecx.stmt_item(DUMMY_SP, item);
        }
    }
    */
}
