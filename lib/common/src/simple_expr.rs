
use syntax::codemap::{Span, DUMMY_SP};


pub trait WriteSimpleExpr {
    fn write_simple_expr(&self, w: &mut SimpleExprWrite);
}

#[derive(Clone, Debug)]
pub enum SimpleExprToken {
    VarReference(String),
    LitString(String),
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

pub trait ToSimpleExprTokens {
    fn to_simple_expr_tokens() -> Vec<SimpleExprToken>;
}

pub trait SimpleExprWrite {
    fn var_reference(&mut self, var_name: &str);
    fn string_lit(&mut self, lit: &str);

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
    use syntax::codemap::{Span, DUMMY_SP};
    use syntax::parse::token::BinOpToken as binops;
    use syntax::ext::base::ExtCtxt;
    use syntax::parse::{token, PResult};
    use syntax::parse::parser::Parser;
    use super::{SimpleExpr, SimpleExprWrite};

    fn parse_expr_into<'cx, 'a>(ecx: &'cx ExtCtxt, mut parser: &mut Parser<'a>, span: Span, w: &mut SimpleExprWrite) -> PResult<'a, ()> {
        loop {
            match parser.token {
                token::BinOp(binops::Plus) => {
                    w.binop_plus();
                },

                token::BinOp(binops::Minus) => {
                    w.binop_minus();
                },

                token::Ident(ref ident) => {
                    w.var_reference(&ident.name.to_string());
                },

                token::CloseDelim(token::Bracket) => {
                    ecx.span_warn(span, "Got close expression - complete");
                    break;
                },

                token::OpenDelim(token::Paren) => {
                    ecx.span_warn(span, "Got close expression - complete");
                    w.open_paren();
                },

                token::CloseDelim(token::Paren) => {
                    ecx.span_warn(span, "Got close expression - complete");
                    w.close_paren();
                },

                _ => {
                    ecx.span_warn(span, "Parsing expression - unknown token");
                }
            }
            parser.bump();
        }

        Ok(())
    }

    pub fn parse_simple_expr<'cx, 'a>(ecx: &'cx ExtCtxt, mut parser: &mut Parser<'a>, span: Span) -> PResult<'a, SimpleExpr> {
        let mut tokens = Vec::new();
        parse_expr_into(ecx, &mut parser, span, &mut tokens);

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

pub mod output_actions {
    use super::{SimpleExpr, SimpleExprToken};
    use output_actions::{OutputAction, IntoOutputActions};
    use syntax::ext::base::ExtCtxt;

    impl IntoOutputActions for SimpleExpr {
        fn into_output_actions<'cx>(&self, ecx: &'cx ExtCtxt) -> Vec<OutputAction> {
            vec![OutputAction::WriteResult(self.clone())]
        }
    }
}

pub mod js_write {
    use super::{SimpleExpr, SimpleExprToken};
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