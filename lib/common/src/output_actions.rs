
use syntax::ast;
use syntax::parse::token;
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult};
use syntax::ext::quote::rt::ToTokens;
use syntax::print::pprust::tts_to_string;
use syntax::tokenstream::TokenTree;

use node::TemplateExpr;
use codegen::IntoWriteStmt;
use jsgen::{IntoJsFunction, IntoJsOutputCall};


pub trait IntoOutputActions {
    fn into_output_actions<'cx>(&self, ecx: &'cx ExtCtxt) -> Vec<OutputAction>;
}

/// Represents a type of action to perform when rendering
#[ignore(dead_code)]
#[derive(Clone, Debug)]
pub enum OutputAction {
    // Text and computed values
    Write(String),
    WriteResult(TemplateExpr),

    // Elements
    WriteOpen(String),
    WriteClose(String),
    WriteVoid(String),
}

impl ToTokens for OutputAction {
    fn to_tokens(&self, ecx: &ExtCtxt) -> Vec<TokenTree> {
        let act = match *self {
            OutputAction::Write(ref contents) => {
                let s = quote_expr!(ecx, $contents.to_owned());
                quote_expr!(ecx, OutputAction::Write($s))
            },

            OutputAction::WriteResult(ref template_expr) => {
                let s = tts_to_string(template_expr.to_tokens(ecx).as_slice());
                quote_expr!(ecx, OutputAction::WriteResult(TemplateExpr($s)))
            },

            OutputAction::WriteOpen(ref element_type) => {
                let s = quote_expr!(ecx, $element_type.to_owned());
                quote_expr!(ecx, OutputAction::WriteOpen($s))
            },

            OutputAction::WriteClose(ref element_type) => {
                let s = quote_expr!(ecx, $element_type.to_owned());
                quote_expr!(ecx, OutputAction::WriteClose($s))
            },

            OutputAction::WriteVoid(ref element_type) => {
                let s = quote_expr!(ecx, $element_type.to_owned());
                quote_expr!(ecx, OutputAction::WriteVoid($s))
            }
        };
        act.to_tokens(ecx)
    }
}

impl IntoWriteStmt for OutputAction {
    fn into_write_stmt<'cx>(&self, ecx: &'cx ExtCtxt, w: ast::Ident) -> ast::Stmt {
        match *self {
            OutputAction::Write(ref contents) => {
                // let w_name =  w.name.to_string();
                let stmt = quote_stmt!(ecx,
                    {
                        println!("Writing contents [{}] to ${}", $contents, "out");
                        write!($w, "{}", $contents);
                    }
                ).unwrap();

                stmt
            },

            // For now, write the expression as a string
            OutputAction::WriteResult(ref template_expr) => {
                let stmt = template_expr.into_write_stmt(ecx, w);

                stmt
            },

            OutputAction::WriteOpen(ref element_type) => {
                let contents = format!("<{}>", element_type);
                let stmt = quote_stmt!(ecx,
                    {
                        println!("Writing contents [{}] to ${}", $contents, "out");
                        write!(out, $contents);
                    }
                ).unwrap();

                stmt
            },

            OutputAction::WriteClose(ref element_type) => {
                let contents = format!("</{}>", element_type);
                let stmt = quote_stmt!(ecx,
                    {
                        println!("Writing contents [{}] to ${}", $contents, "out");
                        write!(out, $contents);
                    }
                ).unwrap();

                stmt
            },

            OutputAction::WriteVoid(ref element_type) => {
                let contents = format!("<{} />", element_type);
                let stmt = quote_stmt!(ecx,
                    {
                        println!("Writing contents [{}] to ${}", $contents, "out");
                        write!(out, $contents);
                    }
                ).unwrap();

                stmt
            }            
        }
    }
}

impl IntoJsOutputCall for OutputAction {
    fn into_js_output_call(&self) -> String {
        match *self {
            OutputAction::Write(ref contents) => {
                format!("IncrementalDOM.text('{}')", contents)
            },

            // For now, write the expression as a string
            OutputAction::WriteResult(ref template_expr) => {
                template_expr.into_js_output_call()
            },

            OutputAction::WriteOpen(ref element_type) => {
                format!("IncrementalDOM.elementOpen('{}')", element_type)
            },

            OutputAction::WriteClose(ref element_type) => {
                format!("IncrementalDOM.elementClose('{}')", element_type)
            },

            OutputAction::WriteVoid(ref element_type) => {
                format!("IncrementalDOM.elementVoid('{}')", element_type)
            }
        }
    }
}
