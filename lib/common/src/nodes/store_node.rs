
use syntax::codemap::Span;
use simple_expr::SimpleExpr;


/// Represents a parsed store definition in template contents
#[derive(Clone, Debug)]
pub struct Store {
    name: String,
    span: Span,
    nodes: Vec<StoreNode>
}

impl Store {
    pub fn with_nodes(span: Span, name: &str, nodes: Vec<StoreNode>) -> Store {
        Store {
            name: name.to_owned(),
            span: span,
            nodes: nodes
        }
    }

    pub fn empty(span: Span, name: &str) -> Store {
        Store {
            name: name.to_owned(),
            span: span,
            nodes: vec![]
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub enum StoreNode {
    // TODO: Define nodes
    DefaultExpr(SimpleExpr),
    ActionExpr(String, SimpleExpr)
}

pub mod parse {
    use super::{Store, StoreNode};
    use syntax::codemap::{Span, DUMMY_SP};
    use syntax::ext::base::ExtCtxt;
    use syntax::parse::{token, PResult};
    use syntax::parse::parser::Parser;
    use simple_expr::SimpleExpr;
    use simple_expr::parse::parse_simple_expr_until;

    fn parse_fat_arrow_expression<'cx, 'a>(ecx: &'cx ExtCtxt, mut parser: &mut Parser<'a>, span: Span) -> PResult<'a, SimpleExpr> {
        try!(parser.expect(&token::FatArrow));

        let simple_expr = try!(parse_simple_expr_until(ecx, &mut parser, span, &|token| token == &token::Semi || token == &token::CloseDelim(token::Brace)));
        if &parser.token != &token::CloseDelim(token::Brace) {
            parser.bump();
        }
        Ok(simple_expr)
    }

    fn parse_action<'cx, 'a>(ecx: &'cx ExtCtxt, mut parser: &mut Parser<'a>, span: Span) -> PResult<'a, StoreNode> {
        let act = try!(parser.parse_ident()).to_string().to_uppercase();
        let simple_expr = try!(parse_fat_arrow_expression(ecx, &mut parser, span));

        Ok(StoreNode::ActionExpr(act.to_owned(), simple_expr))
    }

    fn parse_default<'cx, 'a>(ecx: &'cx ExtCtxt, mut parser: &mut Parser<'a>, span: Span) -> PResult<'a, StoreNode> {
        let simple_expr = try!(parse_fat_arrow_expression(ecx, &mut parser, span));

        Ok(StoreNode::DefaultExpr(simple_expr))
    }

    fn parse_store_contents<'cx, 'a>(ecx: &'cx ExtCtxt, mut parser: &mut Parser<'a>, span: Span) -> PResult<'a, Vec<StoreNode>> {
        let mut nodes: Vec<StoreNode> = Vec::new();

        loop {
            ecx.span_warn(span, &format!("Parsing store - token: {:?}", &parser.token));

            match parser.token {
                token::CloseDelim(token::Brace) => {
                    ecx.span_warn(span, &format!("Parsing store - complete"));
                    parser.bump();
                    break;
                },

                token::Ident(_) => {
                    let ident = try!(parser.parse_ident()).to_string();
                    ecx.span_warn(DUMMY_SP, &format!("Parsing store - ident: {}", ident));

                    match ident {
                        _ if ident == "action" => {
                            let action = try!(parse_action(ecx, parser, span));
                            nodes.push(action);
                            continue;
                        },

                        _ if ident == "default" => {
                            let def = try!(parse_default(ecx, parser, span));
                            nodes.push(def);
                            continue;
                        },

                        _ => {
                            ecx.span_err(span, &format!("Parsing store - unsupported condition label: {}", &ident))
                        }
                    };
                },

                _ => {
                    ecx.span_err(span, &format!("Parsing store - unknown token: {:?}", &parser.token));
                }
            }
            parser.bump();
        }

        Ok(nodes)
    }

    pub fn parse_store<'cx, 'a>(ecx: &'cx ExtCtxt, mut parser: &mut Parser<'a>, span: Span) -> PResult<'a, Store> {
        let store_name = try!(parser.parse_ident());
        ecx.span_warn(DUMMY_SP, &format!("Parsing store - got name: {}", &store_name.to_string()));

        try!(parser.expect(&token::OpenDelim(token::Brace)));

        let nodes = try!(parse_store_contents(ecx, parser, span));
        for node in &nodes {
            ecx.span_warn(DUMMY_SP, &format!("Node: {:?}", node));
        }

        Ok(Store {
            name: store_name.name.to_string(),
            span: span,
            nodes: nodes,
        })
    }
}

pub mod output {
    use super::{Store, StoreNode};
    use syntax::ext::base::ExtCtxt;
    use output_actions::{OutputAction, IntoOutputActions, WriteOutputActions, OutputActionWrite};
    use js_write::{WriteJsFunctions, JsWriteFunctions, WriteJsSwitchBody, JsWriteSwitchBody, WriteJsSimpleExpr};

    impl IntoOutputActions for Store {
        fn into_output_actions(&self) -> Vec<OutputAction> {
            let name = &self.name;
            let nodes = &self.nodes;

            let output_actions = Vec::new();
            /*
            let output_actions: Vec<OutputAction> = nodes.iter()
                .flat_map(|node| node.into_output_actions(ecx))
                .collect();
            */

            output_actions
        }
    }

    impl WriteOutputActions for Store {
        fn write_output_actions(&self, _: &mut OutputActionWrite) {
            // TODO: Implement
        }
    }

    impl WriteJsFunctions for Store {
        fn write_js_functions(&self, funcs: &mut JsWriteFunctions) {
            let store_name = self.name();
            let func_name = format!("rusttemplate_store_template_{}_{}", "main", &store_name);

            funcs.function(&func_name, vec![&store_name, "action"], &|js| {
                js.switch_expr_simple("action.type", &|switch_body| {
                    for node in &self.nodes {
                        node.write_js_switch_body(switch_body);
                    }
                });
            });
        }
    }

    impl WriteJsSwitchBody for StoreNode {
        fn write_js_switch_body(&self, switch: &mut JsWriteSwitchBody) {
            match self {
                &StoreNode::ActionExpr(ref act, ref simple_expr) => {
                    switch.case_str(act, &|js_simple| {
                        simple_expr.write_js_simple_expr(js_simple);
                    });
                },
                &StoreNode::DefaultExpr(ref simple_expr) => {
                    switch.default_case(&|js_simple| {
                        simple_expr.write_js_simple_expr(js_simple);
                    });
                }
            };
        }
    }
}
