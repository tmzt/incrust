
use syntax::codemap::{Span, DUMMY_SP};

use super::view_node::View;
use super::store_node::Store;

#[derive(Clone, Debug)]
pub struct Template {
    name: String,
    span: Span,
    nodes: Vec<TemplateNode>,
}

impl Template {
    pub fn name(&self) -> &str { &self.name }
    pub fn nodes(&self) -> &[TemplateNode] { &self.nodes }
}

// Represents a parsed node in template contents
#[derive(Clone, Debug)]
pub enum TemplateNode {
    ViewNode(String, View),
    StoreNode(String, Store)
    // TODO: RootNode
}

pub mod output {
    use super::{Template, TemplateNode};
    use syntax::ext::base::ExtCtxt;
    use output_actions::{OutputAction, WriteOutputActions, OutputActionWrite};
    use js_write::{WriteJsFunctions, JsWriteFunctions};
    use codegen::lang::{Lang, Html, Js};
    use codegen::named_output::{NamedOutput, NamedOutputType, WriteNamedOutputs, NamedOutputWrite};
    use codegen::output_string_writer::WriteOutputStrings;

    /*
    impl WriteNamedOutputs for Template {
        fn write_named_outputs(&self, w: &mut NamedOutputWrite) {
            for node in &self.nodes {
                node.write_named_outputs(w);
            }
        }
    }

    impl NamedOutput for Template {
        fn output_name(&self) -> &str { &self.name }
        fn output_type(&self) -> NamedOutputType { NamedOutputType::ViewOutput }
    }
    */

    impl<L: Lang> NamedOutput<L> for TemplateNode {
        fn output_name(&self) -> &str {
            match self {
                &TemplateNode::ViewNode(ref view_name, _) => view_name,
                &TemplateNode::StoreNode(ref store_name, _) => store_name
            }
        }

        fn output_type(&self) -> NamedOutputType {
            match self {
                &TemplateNode::ViewNode(_, _) => NamedOutputType::ViewOutput,
                &TemplateNode::StoreNode(_, _) => NamedOutputType::StoreOutput
            }
        }
    }

    impl WriteOutputActions for TemplateNode {
        fn write_output_actions(&self, w: &mut OutputActionWrite) {
            match self {
                &TemplateNode::ViewNode(ref view_name, ref view) => view.write_output_actions(w),
                &TemplateNode::StoreNode(ref store_name, ref store) => store.write_output_actions(w),
            }
        }
    }

    impl WriteOutputActions for Template {
        fn write_output_actions(&self, w: &mut OutputActionWrite) {
            for node in &self.nodes {
                node.write_output_actions(w);
            }
        }
    }

    impl WriteJsFunctions for Template {
        fn write_js_functions(&self, w: &mut JsWriteFunctions) {
            for node in &self.nodes {
                node.write_js_functions(w);
            }
        }
    }

    impl WriteJsFunctions for TemplateNode {
        fn write_js_functions(&self, w: &mut JsWriteFunctions) {
            match self {
                &TemplateNode::ViewNode(ref view_name, ref view) => { view.write_js_functions(w); },
                &TemplateNode::StoreNode(ref store_name, ref store) => { store.write_js_functions(w); },
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
    use nodes::store_node::Store;
    use nodes::store_node::parse::parse_store;
    use simple_expr::SimpleExpr;
    use simple_expr::parse::parse_simple_expr;

    enum TemplateNodeType {
        ViewNode,
        StoreNode
    }

    fn parse_template_node_type<'a>(parser: &mut Parser<'a>) -> PResult<'a, TemplateNodeType> {
            let ident = (parser.parse_ident()?).to_string();
            parser.span_warn(parser.span, &format!("Parsing template - ident: {}", ident));

            match ident {
                _ if ident == "view" => Ok(TemplateNodeType::ViewNode),
                _ if ident == "store" => Ok(TemplateNodeType::StoreNode),

                _ => {
                    Err(parser.span_fatal(parser.prev_span, &format!("Parsing template - unsupported node type: {}", &ident)))
                }
            }
    }

    pub fn parse_template<'cx, 'a>(ecx: &'cx ExtCtxt<'a>, parser: &mut Parser<'a>, span: Span, name: &str) -> PResult<'a, Template> {
        let mut nodes: Vec<TemplateNode> = vec![];

        loop {
            parser.span_warn(parser.span, &format!("Parsing template - got token: {:?}", &parser.token));

            match parser.token {
                token::CloseDelim(token::Bracket) => {
                    parser.span_warn(parser.span, "Parsing template - got closing bracket");
                    break;
                },

                token::Ident(_) => {
                    let node_type = parse_template_node_type(parser)?;
                    match node_type {
                        TemplateNodeType::ViewNode => {
                            parser.span_warn(span, "Parsing view");

                            let sp = parser.span.clone();
                            let view = parse_view(ecx, parser, sp)?;
                            let view_name = view.name().to_owned();
                            nodes.push(TemplateNode::ViewNode(view_name, view));
                        },

                        TemplateNodeType::StoreNode => {
                            parser.span_warn(span, "Parsing store");
                            let store = parse_store(ecx, parser)?;
                            let store_name = store.name().to_owned();
                            nodes.push(TemplateNode::StoreNode(store_name, store));
                        }
                    };
                },

                _ => {
                    return Err(parser.span_fatal(span, &format!("Parsing template - got unexpected token: {:?}", parser.token)));
                }
            }
        }

        Ok(Template {
            name: name.to_owned(),
            span: span,
            nodes: nodes
        })
    }

}
