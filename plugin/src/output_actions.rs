
use syntax::ast;
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult};

use node::TemplateExpr;
use codegen::IntoWriteStmt;


pub trait IntoOutputActions {
    fn into_output_actions<'cx>(&self, ecx: &'cx ExtCtxt) -> Vec<OutputAction>;
}

// Represents a type of action to perform when rendering
pub enum OutputAction {
    // Text and computed values
    Write(String),
    WriteResult(TemplateExpr),

    // Elements
    WriteOpen(String),
    WriteClose(String),
    WriteVoid(String)

    //CallTemplate(String),
    //WriteElement(Element),
}




impl IntoWriteStmt for OutputAction {
    fn into_write_stmt<'cx>(&self, ecx: &'cx ExtCtxt, w: ast::Ident) -> ast::Stmt {
        match *self {
            OutputAction::Write(ref contents) => {
                //let w_name =  w.name.to_string();
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
