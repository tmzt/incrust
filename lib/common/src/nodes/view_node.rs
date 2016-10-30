
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
    use output_actions::{OutputAction, IntoOutputActions};

    impl IntoOutputActions for View {
        fn into_output_actions<'cx>(&self, ecx: &'cx ExtCtxt) -> Vec<OutputAction> {
            let name = &self.name;
            let nodes = &self.nodes;

            let output_actions: Vec<OutputAction> = nodes.iter()
                .flat_map(|node| node.into_output_actions(ecx))
                .collect();

            output_actions
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
    use codegen::WriteItems;

    /*
    fn create_view_item<'cx>(ecx: &'cx ExtCtxt, view: &View) -> P<ast::Item> {
        let name = ecx.ident_of(&format!("rusttemplate_view_{}", view.name));
        let block = view.into_block(ecx);

        let inputs = vec![];
        let ret_ty = quote_ty!(ecx, String);
        ecx.item_fn(DUMMY_SP, name, inputs, ret_ty, block)
    }
    */

    /*
    fn create_view_item<'cx>(ecx: &'cx ExtCtxt, view: &View) -> P<ast::Item> {
    }
    */

    impl WriteItems for View {
        fn write_items<'cx>(&self, ecx: &'cx mut ExtCtxt) {
            //let item = create_view_item(ecx, &self);
            //ecx.stmt_item(DUMMY_SP, item);
        }
    }

    /*
    impl IntoBlock for View {
        fn into_block<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Block> {
            let mut stmts: Vec<ast::Stmt> = vec![];
            view.write_strings()
            view.write_string_output_stmts()
        }
    }
    */

    /*
    impl WriteItems for View {
        fn write_items<'cx>(&self, ecx: &'cx mut ExtCtxt) {
            let mut output_actions: Vec<OutputAction> = vec![];
            for node in &self.nodes {
                node.write_output_actions(output_actions);
                node.write_items(ecx);
            }
        }
    }
    */
}

mod codegen {

    /*
    fn create_view_item_stmts<'cx>(ecx: &'cx ExtCtxt, views: &[View]) -> Vec<ast::Stmt> {
        let view_item_stmts: Vec<ast::Stmt> = views.iter()
            .map(|view| view.into_view_item(ecx))
            .map(|item| ecx.stmt_item(DUMMY_SP, item))
            .collect();

        view_item_stmts
    }
    */
}

/*
mod output_items {
    use super::View;
    use syntax::ast;
    use syntax::codemap::{Span, DUMMY_SP};
    use syntax::ext::base::ExtCtxt;
    use syntax::ext::quote::rt::ToTokens;
    use syntax::ptr::P;
    use syntax::ext::build::AstBuilder;

    use codegen::{IntoWriteStmt, IntoViewItem, IntoBlock};
    use output_actions::{OutputAction, IntoOutputActions};

    fn create_view_item<'cx>(ecx: &'cx ExtCtxt, view: &View) -> P<ast::Item> {
        let name = ecx.ident_of(&format!("rusttemplate_view_{}", view.name));
        let block = view.into_block(ecx);

        let inputs = vec![];
        let ret_ty = quote_ty!(ecx, String);
        ecx.item_fn(DUMMY_SP, name, inputs, ret_ty, block)
    }

    impl IntoViewItem for View {
        fn into_view_item<'cx>(&self, ecx: &'cx ExtCtxt) -> P<ast::Item> {
            create_view_item(ecx, &self)
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
}
*/
