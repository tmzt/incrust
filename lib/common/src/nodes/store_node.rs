
use syntax::codemap::Span;
use value::Value;
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
    DefaultExpr(Value),
    ActionExpr(String, Value)
}


pub mod parse {
    use super::{Store, StoreNode};
    use syntax_pos::mk_sp;
    use syntax::codemap::{Span, DUMMY_SP};
    use syntax::ext::base::ExtCtxt;
    use syntax::parse::{token, PResult};
    use syntax::parse::parser::Parser;
    use syntax::parse::common::SeqSep;
    use value::Value;
    use value::parse::parse_value;
    use simple_expr::SimpleExpr;
    use simple_expr::parse::parse_simple_expr_until;
    use object_expr::parse::parse_object_expr;

    enum StoreArmType {
        DefaultArm,
        ActionArm,
    }

    /*
    fn parse_fat_arrow_expression<'cx, 'a>(ecx: &'cx ExtCtxt<'a>, parser: &mut Parser<'a>, span: Span) -> PResult<'a, Value> {
        try!(parser.expect(&token::FatArrow));

        let expr_tts = parser.parse_seq_to_before_end(&token::CloseDelim(token::Brace), SeqSep::trailing_allowed(token::Semi), |pp| pp.parse_token_tree());
        let mut expr_parser = ecx.new_parser_from_tts(&expr_tts);

        //let simple_expr = try!(parse_simple_expr_until(ecx, &mut parser, span, &|token| token == &token::Semi || token == &token::CloseDelim(token::Brace)));
        let value = try!(parse_value(ecx, parser, span));
        if &parser.token != &token::CloseDelim(token::Brace) {
            parser.bump();
        }
        Ok(value)
    }

    fn parse_action<'cx, 'a>(ecx: &'cx ExtCtxt<'a>, parser: &mut Parser<'a>, span: Span) -> PResult<'a, StoreNode> {
        let act = try!(parser.parse_ident()).to_string().to_uppercase();
        ecx.span_warn(span, &format!("Parsing store - got action {}", &act));

        let value = try!(parse_fat_arrow_expression(ecx, parser, span));

        Ok(StoreNode::ActionExpr(act.to_owned(), value))
    }

    fn parse_default<'cx, 'a>(ecx: &'cx ExtCtxt<'a>, parser: &mut Parser<'a>, span: Span) -> PResult<'a, StoreNode> {
        let value = try!(parse_fat_arrow_expression(ecx, parser, span));

        Ok(StoreNode::DefaultExpr(value))
    }

    fn parse_store_contents<'cx, 'a>(ecx: &'cx ExtCtxt<'a>, parser: &mut Parser<'a>, span: Span) -> PResult<'a, Vec<StoreNode>> {
        let mut nodes: Vec<StoreNode> = Vec::new();

        //let body_tts = parser.parse_seq_to_before_end(&token::CloseDelim(token::Brace), SeqSep::trailing_allowed(token::Semi), |pp| pp.parse_token_tree());

        loop {
            ecx.span_warn(span, &format!("Parsing store - token: {:?}", &parser.token));

            match parser.token {
                token::Eof => {
                    ecx.span_warn(span, &format!("Parsing store - complete"));
                    break;                    
                },

                token::Ident(_) => {
                    let ident = try!(parser.parse_ident()).to_string();
                    ecx.span_warn(DUMMY_SP, &format!("Parsing store - ident: {}", ident));

                    let mut entry_parser = {
                        let entry_tts = parser.parse_seq_to_before_end(&token::CloseDelim(token::Brace),
                            SeqSep::trailing_allowed(token::Semi), |pp| pp.parse_token_tree());
                        ecx.new_parser_from_tts(&entry_tts)
                    };

                    match ident {
                        _ if ident == "action" => {
                            let action = try!(parse_action(ecx, &mut entry_parser, span));
                            nodes.push(action);
                            continue;
                        },

                        _ if ident == "default" => {
                            let def = try!(parse_default(ecx, &mut entry_parser, span));
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

    fn parse_fat_arrow_expression<'cx, 'a>(ecx: &'cx ExtCtxt<'a>, parser: &mut Parser<'a>, span: Span) -> PResult<'a, Value> {
        try!(parser.expect(&token::FatArrow));

        let expr_tts = parser.parse_seq_to_before_end(&token::CloseDelim(token::Brace), SeqSep::trailing_allowed(token::Semi), |pp| pp.parse_token_tree());
        let mut expr_parser = ecx.new_parser_from_tts(&expr_tts);

        //let simple_expr = try!(parse_simple_expr_until(ecx, &mut parser, span, &|token| token == &token::Semi || token == &token::CloseDelim(token::Brace)));
        let value = try!(parse_value(ecx, parser, span));
        if &parser.token != &token::CloseDelim(token::Brace) {
            parser.bump();
        }
        Ok(value)
    }
    */

    fn parse_store_arm_keyword<'cx, 'a>(ecx: &'cx ExtCtxt<'a>, parser: &mut Parser<'a>) -> PResult<'a, StoreArmType> {
            let ident = try!(parser.parse_ident()).to_string();
            ecx.span_warn(DUMMY_SP, &format!("Parsing store - ident: {}", ident));

            match ident {
                _ if ident == "action" => Ok(StoreArmType::ActionArm),
                _ if ident == "default" => Ok(StoreArmType::DefaultArm),

                _ => {
                    Err(parser.span_fatal(parser.prev_span, &format!("Parsing store - unsupported condition label: {}", &ident)))
                }
            }
    }

    fn parse_action_str<'cx, 'a>(ecx: &'cx ExtCtxt<'a>, parser: &mut Parser<'a>) -> PResult<'a, String> {
        let ident = try!(parser.parse_ident()).to_string().to_owned();
        parser.warn(&format!("Parsing store - ident: {}", &ident));
        Ok(ident)
    }

    fn parse_store_arm<'cx, 'a>(ecx: &'cx ExtCtxt<'a>, parser: &mut Parser<'a>) -> PResult<'a, StoreNode> {
        let arm_type = parse_store_arm_keyword(ecx, parser)?;
        let arm = match arm_type {
            StoreArmType::ActionArm => {
                let action_str = parse_action_str(ecx, parser)?;
                parser.expect(&token::FatArrow)?;
                let value = parse_value(ecx, parser)?;

                Ok(StoreNode::ActionExpr(action_str, value))
            },

            StoreArmType::DefaultArm => {
                parser.expect(&token::FatArrow)?;
                let value = parse_value(ecx, parser)?;

                Ok(StoreNode::DefaultExpr(value))
            }
        };

        //parser.eat(&token::Semi);
        arm
    }

    pub fn parse_store<'cx, 'a>(ecx: &'cx ExtCtxt<'a>, parser: &mut Parser<'a>) -> PResult<'a, Store> {
        let store_name = try!(parser.parse_ident()).to_string();
        let lo = parser.span.lo;
        parser.span_warn(parser.span, &format!("Parsing store - got name: {}", &store_name));

        //try!(parser.expect(&token::OpenDelim(token::Brace)));

        let nodes = parser.parse_unspanned_seq(
            &token::OpenDelim(token::Brace),
            &token::CloseDelim(token::Brace),
            SeqSep::trailing_allowed(token::Semi),
            |p| parse_store_arm(ecx, p)
        )?;

        Ok(Store {
            name: store_name,
            span: mk_sp(lo, parser.span.hi),
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
                &StoreNode::ActionExpr(ref act, ref value) => {
                    switch.case_str(act, &|v| {
                        v.write_value(value);
                    });
                },
                &StoreNode::DefaultExpr(ref value) => {
                    switch.default_case(&|v| {
                        v.write_value(value);
                    });
                }
            };
        }
    }
}
