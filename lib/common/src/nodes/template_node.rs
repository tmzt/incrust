
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
    use output::ToData;
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

    /*
    impl ToData<String, Html> for Template {
        fn to_data<'cx>(&self, ecx: &'cx ExtCtxt) -> String {
            let mut html = String::new();
            &self.write_output_strings(ecx, &mut html);
            html
        }
    }
    */
}

pub mod output_ast {
    use super::{Template, TemplateNode};
    use codegen::lang::{Lang, Html, Js};
    use codegen::output_item_writer::{WriteOutputItems, OutputItemWrite};
    use syntax::ext::base::{ExtCtxt, MacResult, MacEager, TTMacroExpander};

    /*
    impl<L: Lang> WriteOutputItems<L> for Template {
        fn write_output_items<'cx>(&self, ecx: &'cx ExtCtxt, w: &mut OutputItemWrite<L>) {

        }
    }

    impl<L: Lang> WriteOutputItems<L> for TemplateNode {
        fn write_output_items<'cx>(&self, ecx: &'cx ExtCtxt, w: &mut OutputItemWrite<L>) {
            match self {
                &TemplateNode::ViewNode(_, ref view) => { view.write_output_items(ecx, w); },
                &TemplateNode::StoreNode(_, ref store) => { store.write_output_items(ecx, w); },
            }
        }
    }
    */

    /*
    impl<L: Lang> WriteOutputItems<L> for ViewNode {
        fn write_output_items<'cx>(&self, ecx: &'cx ExtCtxt, w: &mut OutputItemWrite<L>) {
        }
    }

    impl<L: Lang> WriteOutputItems<L> for StoreNode {
        fn write_output_items<'cx>(&self, ecx: &'cx ExtCtxt, w: &mut OutputItemWrite<L>) {
        }
    }
    */

    /*
    impl WriteItems for Template {
        fn write_items<'cx>(&self, ecx: &'cx ExtCtxt, w: &mut ItemWrite) {
            for node in &self.nodes {
                node.write_items(ecx, w);
            }
        }
    }

    impl WriteItems for TemplateNode {
        fn write_items<'cx>(&self, ecx: &'cx ExtCtxt, w: &mut ItemWrite) {
            match self {
                &TemplateNode::ViewNode(_, ref view) => {
                    view.write_items(ecx, w);
                },
                _ => {
                }
            }
        }
    }
    */
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
    use simple_expr::SimpleExpr;
    use simple_expr::parse::parse_simple_expr;

    pub fn parse_template<'cx, 'a>(ecx: &'cx ExtCtxt, mut parser: &mut Parser<'a>, span: Span, name: &str) -> PResult<'a, Template> {
        let mut nodes = Vec::new();

        loop {
            ecx.span_warn(span, &format!("Parsing template - got token: {:?}", &parser.token));

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
                            nodes.push(TemplateNode::ViewNode("root".to_owned(), view));
                        },

                        "store" => {
                            ecx.span_warn(span, "Parsing store");

                            // TODO: Expand the syntax we can parse
                            let store_ident = try!(parser.parse_ident());
                            let store_name = store_ident.to_string();
                            let store = Store::empty(span, &store_name);
                            ecx.span_warn(span, &format!("Parsing store - got: {:?}", store));

                            nodes.push(TemplateNode::StoreNode(store_name.to_owned(), store));
                        }

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
