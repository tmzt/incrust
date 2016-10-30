
use syntax::codemap::{Span, DUMMY_SP};

use super::view_node::View;
use super::store_node::Store;

#[derive(Clone, Debug)]
pub struct Template {
    name: String,
    span: Span,
    nodes: Vec<TemplateNode>,
}

// Represents a parsed node in template contents
#[derive(Clone, Debug)]
pub enum TemplateNode {
    ViewNode(String, View),
    StoreNode(String, Store)
    // TODO: RootNode
}

pub mod output {
    use super::TemplateNode;
    use syntax::ext::base::ExtCtxt;
    use output_actions::{OutputAction, IntoOutputActions};

    impl IntoOutputActions for TemplateNode {
        fn into_output_actions<'cx>(&self, ecx: &'cx ExtCtxt) -> Vec<OutputAction> {
            match self {
                &TemplateNode::ViewNode(ref view_name, ref view) => view.into_output_actions(ecx),
                &TemplateNode::StoreNode(ref store_name, ref store) => store.into_output_actions(ecx),
            }
        }
    }

    // TODO: Output JS
}

pub mod output_ast {
    use super::{Template, TemplateNode};
    use codegen::{WriteItems};
    use syntax::ext::base::{ExtCtxt, MacResult, MacEager, TTMacroExpander};

    impl WriteItems for Template {
        fn write_items<'cx>(&self, ecx: &'cx mut ExtCtxt) {
            for node in &self.nodes {
                node.write_items(ecx);
            }
        }
    }

    impl WriteItems for TemplateNode {
        fn write_items<'cx>(&self, ecx: &'cx mut ExtCtxt) {
            match self {
                &TemplateNode::ViewNode(_, ref view) => {
                    view.write_items(ecx);;
                },
                _ => {
                }
            }
        }
    }
}

// NEXTREV: Make this depend on syntax/syntex
pub mod expand {
    use super::Template;
    use syntax::tokenstream::TokenTree;
    use syntax::codemap::{Span, DUMMY_SP};
    use syntax::util::small_vector::SmallVector;
    use syntax::ext::base::{ExtCtxt, MacResult, MacEager, TTMacroExpander};

    impl TTMacroExpander for Template {
        fn expand<'cx>(&self, ecx: &'cx mut ExtCtxt, _: Span, tts: &[TokenTree]) -> Box<MacResult + 'cx> {
            let mut parser = ecx.new_parser_from_tts(tts);
            let w_ident = parser.parse_ident().unwrap();

            // TODO: Make this work
            //self.write_string_output_stmts(ecx, ecx);

            /*
            let w_ident = parser.parse_ident().unwrap();
            codegen::create_template_write_block(ecx, w_ident, &self.compiled_views)
            */

            // Empty (but must consist of items)
            MacEager::items(SmallVector::zero())
        }
    }
}

pub mod parse {
    use super::{Template, TemplateNode};
    use syntax::tokenstream::TokenTree;
    use syntax::codemap::{Span, DUMMY_SP};
    use syntax::ext::base::ExtCtxt;
    use syntax::ext::quote::rt::ToTokens;
    use syntax::parse::{token, PResult};
    use syntax::parse::parser::Parser;

    use output_actions::{OutputAction, IntoOutputActions};
    use nodes::view_node::{View};
    use nodes::view_node::parse::parse_view;
    use simple_expr::SimpleExpr;
    use simple_expr::parse::parse_simple_expr;

    pub fn parse_template<'cx, 'a>(ecx: &'cx ExtCtxt, mut parser: &mut Parser<'a>, span: Span, name: &str) -> PResult<'a, Template> {
        let mut nodes = Vec::new();

        loop {
            match parser.token {
                token::CloseDelim(token::Bracket) => {
                    ecx.span_warn(span, "Parsing template - got closing bracket");
                    break;
                },

                token::Ident(_) => {
                    let keyword_token = try!(parser.parse_ident());
                    let keyword = keyword_token.name.to_string().to_owned();

                    ecx.span_warn(span, &format!("Parsing template - got keyword: {:?}", &keyword));
                    match keyword.as_ref() {
                        "view" => {
                            ecx.span_warn(span, "Parsing view");
                            let view = try!(parse_view(ecx, &mut parser, span));
                        },

                        _ => {
                            ecx.span_warn(span, &format!("Parsing template - got other keyword: {:?}", keyword));
                        }
                    }

                    /*
                    let keyword = if let token::Ident(ref ident) = parser.token {
                        let keyword = ident.name.to_string().to_owned();
                        ecx.span_warn(span, &format!("Parsing template contents - got keyword: {:?}", &keyword));
                        keyword
                    };
                    */

                    /*
                    let keyword = parser.token.name.to_string().to_owned();

                    ecx.span_warn(span, &format!("Parsing expression - got keyword: {:?}", &keyword));
                    match keyword.as_ref() {
                        "view" => {
                            ecx.span_warn(span, "Parsing view");
                        },

                        _ => {
                            ecx.span_warn(span, &format!("Parsing element: {:?}", keyword));
                        }
                    }
                    */
                },

                _ => {
                    ecx.span_err(span, &format!("Parsing template - got unexpected token: {:?}", parser.token));
                }
            }
        }

        let template = Template { name: name.to_owned(), span: span, nodes: nodes };
        Ok(template)
    }

}
