
use syntax::codemap::{Span, DUMMY_SP};


pub trait WriteSimpleExpr {
    fn write_simple_expr(&self, w: &mut SimpleExprWrite);
}

#[derive(Clone, Debug)]
pub enum SimpleExprNumber {
    Int64(i64),
    Int32(i32)
}

#[derive(Clone, Debug)]
pub enum SimpleExprToken {
    VarReference(String),
    LitString(String),
    LitNumber(SimpleExprNumber),
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    BinopPlus,
    BinopMinus
}

#[derive(Clone, Debug)]
pub struct SimpleExpr {
    span: Span,
    tokens: Vec<SimpleExprToken>
}

impl SimpleExpr {
    pub fn tokens(&self) -> &[SimpleExprToken] {
        &self.tokens
    }
}

pub trait ToSimpleExprTokens {
    fn to_simple_expr_tokens() -> Vec<SimpleExprToken>;
}

pub trait SimpleExprWrite {
    fn var_reference(&mut self, var_name: &str);
    fn string_lit(&mut self, lit: &str);
    fn number_lit(&mut self, lit: &SimpleExprNumber);

    fn open_brace(&mut self);
    fn close_brace(&mut self);

    fn open_paren(&mut self);
    fn close_paren(&mut self);

    fn binop_plus(&mut self);
    fn binop_minus(&mut self);
}

impl SimpleExprWrite for Vec<SimpleExprToken> {
    fn var_reference(&mut self, var_name: &str) {
        self.push(SimpleExprToken::VarReference(var_name.to_owned()));
    }

    fn string_lit(&mut self, lit: &str) {
        self.push(SimpleExprToken::LitString(lit.to_owned()));
    }

    fn number_lit(&mut self, lit: &SimpleExprNumber) {
        self.push(SimpleExprToken::LitNumber(lit.to_owned()));
    }

    fn open_brace(&mut self) {
        self.push(SimpleExprToken::OpenBrace);
    }

    fn close_brace(&mut self) {
        self.push(SimpleExprToken::CloseBrace);
    }

    fn open_paren(&mut self) {
        self.push(SimpleExprToken::OpenParen);
    }

    fn close_paren(&mut self) {
        self.push(SimpleExprToken::CloseParen);
    }

    fn binop_plus(&mut self) {
        self.push(SimpleExprToken::BinopPlus);
    }

    fn binop_minus(&mut self) {
        self.push(SimpleExprToken::BinopMinus);
    }
}

pub mod parse {
    use syntax::ast::{self, LitKind, LitIntType, IntTy};
    use syntax::codemap::{Span, DUMMY_SP};
    use syntax::parse::token::DelimToken;
    use syntax::parse::token::BinOpToken as binops;
    use syntax::parse::token::Lit as literals;
    use syntax::ext::base::ExtCtxt;
    use syntax::parse::{token, PResult};
    use syntax::parse::parser::Parser;
    use super::{SimpleExpr, SimpleExprToken, SimpleExprNumber, SimpleExprWrite};

    fn parse_var_reference<'cx, 'a>(ecx: &'cx ExtCtxt, mut parser: &mut Parser<'a>, span: Span) -> PResult<'a, SimpleExprToken> {
        // NEXTREV: Add JsPathExpr variant

        let mut var_name = String::new();
        loop {
            ecx.span_warn(span, &format!("Parsing var reference - token: {:?}", &parser.token));

            match parser.token {
                token::Ident(ref ident) => {
                    var_name.push_str(&ident.name.to_string());
                },
                token::Dot => {
                    var_name.push('.');
                },
                _ => {
                    ecx.span_warn(span, "Invalid token in VarReference - complete");
                    break;
                }
            }
            parser.bump();
        }
        ecx.span_warn(span, &format!("Parsing var reference - complete: {:?}", &var_name));
        Ok(SimpleExprToken::VarReference(var_name.to_owned()))
    }

    fn parse_expr_contents_into_until<'cx, 'a>(ecx: &'cx ExtCtxt, mut parser: &mut Parser<'a>, span: Span, w: &mut SimpleExprWrite, end_cond: &Fn(&token::Token) -> bool) -> PResult<'a, ()> {
        loop {
            ecx.span_warn(span, &format!("Parsing expression contents - token: {:?}", &parser.token));

            if end_cond(&parser.token) {
                    ecx.span_warn(span, &format!("Got close [{:?}] - completed expression", &parser.token));
                    break;
            }

            match parser.token {
                token::Ident(_) => {
                    ecx.span_warn(span, &format!("Got ident - parsing var reference"));
                    
                    match parse_var_reference(ecx, parser, span) {
                        Ok(SimpleExprToken::VarReference(ref var_name)) => {
                            w.var_reference(var_name);
                        },
                        _ => { ecx.span_warn(span, "Unable to parse VarReference for this ident"); }
                    };
                    // Don't bump
                    continue;
                },

                token::Literal(_, _) => {
                    let lit = try!(parser.parse_lit());
                    match &lit.node {
                        &LitKind::Str(ref s, _) => {
                            let string_value = s.to_string();
                            ecx.span_warn(span, &format!("Parsing simple expression - got literal string: {}", &string_value));
                            w.string_lit(&string_value);
                        },

                        &LitKind::Int(n, int_ty) => {
                            ecx.span_warn(span, &format!("Parsing simple expression - got literal int ({:?}): {}", int_ty, n));
                            match int_ty {
                                LitIntType::Signed(IntTy::I64) => {
                                    w.number_lit(&SimpleExprNumber::Int64(n as i64));
                                },

                                LitIntType::Signed(IntTy::I32) => {
                                    w.number_lit(&SimpleExprNumber::Int32(n as i32));
                                },

                                LitIntType::Unsuffixed => {
                                    // TODO: Determine what we should do with this
                                    w.number_lit(&SimpleExprNumber::Int64(n as i64));
                                },

                                _ => {
                                    ecx.span_warn(span, &format!("Parsing simple expression - got unsupported number ({:?}): {:?}", int_ty, n));
                                }
                            }
                        },

                        _ => {
                            ecx.span_warn(span, &format!("Parsing simple expression - got unsupported literal: {:?}", &lit.node));
                        }
                    }
                    // Don't bump, as we already parsed the literal
                    continue;
                },

                token::BinOp(binops::Plus) => {
                    w.binop_plus();
                },

                token::BinOp(binops::Minus) => {
                    w.binop_minus();
                },

                token::OpenDelim(token::Paren) => {
                    w.open_paren();
                },

                token::CloseDelim(token::Paren) => {
                    w.close_paren();
                },

                _ => {
                    ecx.span_err(span, &format!("Parsing simple expression - unknown token: {:?}", &parser.token));
                }
            }
            parser.bump();
        }

        Ok(())
    }

    pub fn parse_simple_expr_until<'cx, 'a>(ecx: &'cx ExtCtxt, mut parser: &mut Parser<'a>, span: Span, end_cond: &Fn(&token::Token) -> bool) -> PResult<'a, SimpleExpr> {
        let mut tokens = Vec::new();
        try!(parse_expr_contents_into_until(ecx, &mut parser, span, &mut tokens, end_cond));

        let simple_expr = SimpleExpr { span: span, tokens: tokens };
        Ok(simple_expr)
    }

    pub fn parse_simple_expr<'cx, 'a>(ecx: &'cx ExtCtxt, mut parser: &mut Parser<'a>, span: Span, end_delim: DelimToken) -> PResult<'a, SimpleExpr> {
        let mut tokens = Vec::new();
        try!(parse_expr_contents_into_until(ecx, &mut parser, span, &mut tokens, &|token| token == &token::CloseDelim(end_delim)));

        let simple_expr = SimpleExpr { span: span, tokens: tokens };
        Ok(simple_expr)
    }
}

pub mod output_ast {
    use syntax::ast;
    use syntax::codemap::{Span, DUMMY_SP};
    use syntax::ext::base::ExtCtxt;
    use syntax::ext::quote::rt::ToTokens;
    use syntax::tokenstream::TokenTree;
    use syntax::parse::token::BinOpToken as binops;
    use syntax::parse::{token, PResult};

    use super::{SimpleExpr, SimpleExprToken};
    use js_write::{WriteJs, WriteJsSimpleExpr, JsWriteSimpleExpr};
    use codegen::IntoWriteStmt;

    impl ToTokens for SimpleExpr {
        fn to_tokens(&self, ecx: &ExtCtxt) -> Vec<TokenTree> {
            let mut tokens: Vec<TokenTree> = vec![];
            for token in &self.tokens {
                tokens.push(TokenTree::Token(DUMMY_SP, token::BinOp(binops::Plus)));
            }
            tokens
        }
    }

    // Used to output expression as a write statment
    impl IntoWriteStmt for SimpleExpr {
        fn into_write_stmt<'cx>(&self, ecx: &'cx ExtCtxt, w: ast::Ident) -> ast::Stmt {
            let mut contents = String::new();
            &self.write_js_simple_expr(&mut contents);

            let stmt = quote_stmt!(ecx, {
                    println!("Writing contents [{}] to ${}", $contents, "out");
                    write!(out, $contents);
                }).unwrap();

            stmt
        }
    }
}

mod output_strings {
    use super::{SimpleExpr, SimpleExprToken, SimpleExprNumber};
    use syntax::codemap::DUMMY_SP;
    use syntax::ext::base::ExtCtxt;
    use codegen::lang::{Lang, Js, Html};
    use codegen::output_string_writer::{WriteOutputStrings, OutputStringWrite};
    use output_actions::OutputAction;

    impl WriteOutputStrings<Html> for SimpleExpr {
        fn write_output_strings<'s, 'cx>(&self, ecx: &'cx ExtCtxt, w: &'s mut OutputStringWrite<Html>) {
            let mut s = String::new();
            for token in &self.tokens {
                ecx.span_warn(DUMMY_SP, &format!("Writing token: {:?}", &token));
                match token {
                    &SimpleExprToken::VarReference(ref var_name) => {
                        w.write_output_string(ecx, &format!("{}", var_name));
                    },

                    &SimpleExprToken::LitString(ref contents) => {
                        w.write_output_string(ecx, &format!("\"{}\"", contents));
                    },

                    &SimpleExprToken::LitNumber(ref contents) => {
                        let s = match contents {
                            &SimpleExprNumber::Int64(n) => format!("{}", n),
                            &SimpleExprNumber::Int32(n) => format!("{}", n)
                        };
                        w.write_output_string(ecx, &s);
                    },

                    &SimpleExprToken::OpenBrace => {
                        w.write_output_string(ecx, &format!("{{"));
                    },

                    &SimpleExprToken::CloseBrace => {
                        w.write_output_string(ecx, &format!("}}"));
                    },

                    &SimpleExprToken::OpenParen => {
                        w.write_output_string(ecx, &format!("("));
                    },

                    &SimpleExprToken::CloseParen => {
                        w.write_output_string(ecx, &format!(")"));
                    },

                    &SimpleExprToken::BinopPlus => {
                        w.write_output_string(ecx, &format!("+"));
                    },

                    &SimpleExprToken::BinopMinus => {
                        w.write_output_string(ecx, &format!("-"));
                    }
                }
            }
        }
    }
}

pub mod js_write {
    use super::{SimpleExpr, SimpleExprToken, SimpleExprNumber};
    use js_write::{WriteJs, WriteJsSimpleExpr, JsWriteSimpleExpr};

    impl WriteJsSimpleExpr for SimpleExpr {
        fn write_js_simple_expr(&self, js: &mut JsWriteSimpleExpr) {
            for token in &self.tokens {
                match token {
                    &SimpleExprToken::VarReference(ref var_name) => {
                        js.var_reference(var_name);
                    },

                    &SimpleExprToken::LitString(ref contents) => {
                        js.string_lit(contents);
                    },

                    &SimpleExprToken::LitNumber(ref contents) => {
                        match contents {
                            &SimpleExprNumber::Int64(n) => { js.int64_lit(n); },
                            &SimpleExprNumber::Int32(n) => { js.int32_lit(n); }
                        };
                    },

                    &SimpleExprToken::OpenBrace => {
                        js.open_brace();
                    },

                    &SimpleExprToken::CloseBrace => {
                        js.close_brace();
                    },

                    &SimpleExprToken::OpenParen => {
                        js.open_paren();
                    },

                    &SimpleExprToken::CloseParen => {
                        js.close_paren();
                    },

                    &SimpleExprToken::BinopPlus => {
                        js.binop_plus();
                    },

                    &SimpleExprToken::BinopMinus => {
                        js.binop_minus();
                    }
                };
            }
        }   
    }
}