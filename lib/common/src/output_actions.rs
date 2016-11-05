
use syntax::ast;
use syntax::parse::token;
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult};
use syntax::ext::quote::rt::ToTokens;
use syntax::print::pprust::tts_to_string;
use syntax::tokenstream::TokenTree;

use codegen::IntoWriteStmt;

use simple_expr::{SimpleExpr, js_write};
use js_write::{WriteJs, JsWrite, WriteJsSimpleExpr};


pub trait WriteOutputActions {
    fn write_output_actions(&self, w: &mut OutputActionWrite);
}

pub trait OutputActionWrite {
    fn write_output_action(&mut self, output_action: &OutputAction);
}

pub trait IntoOutputActions {
    fn into_output_actions(&self) -> Vec<OutputAction>;
}

/// Represents a type of action to perform when rendering
#[derive(Clone, Debug)]
pub enum OutputAction {
    // Text and computed values
    Write(String),
    WriteResult(SimpleExpr),

    // Elements
    WriteOpen(String),
    WriteClose(String),
    WriteVoid(String),
}

mod output_strings {
    use super::{OutputAction, WriteOutputActions};
    use syntax::ext::base::ExtCtxt;
    use codegen::lang::{Lang, Js, Html};
    use codegen::output_string_writer::{WriteOutputStrings, OutputStringWrite};

    impl<S: WriteOutputActions> WriteOutputStrings<Html> for S {
        fn write_output_strings<'s, 'cx>(&self, ecx: &'cx ExtCtxt, w: &'s mut OutputStringWrite<Html>) {
            let mut output_actions = Vec::new();
            self.write_output_actions(&mut output_actions);
            for output_action in &output_actions {
                output_action.write_output_strings(ecx, w);
            }
        }
    }

    impl WriteOutputStrings<Html> for OutputAction {
        fn write_output_strings<'s, 'cx>(&self, ecx: &'cx ExtCtxt, w: &'s mut OutputStringWrite<Html>) {
            match self {
                &OutputAction::Write(ref contents) => {
                    w.write_output_string(ecx, &contents);
                },

                &OutputAction::WriteResult(ref simple_expr) => {
                    &simple_expr.write_output_strings(ecx, w);
                },

                &OutputAction::WriteOpen(ref element_type) => {
                    w.write_output_string(ecx, &format!("<{}>", &element_type));
                },

                &OutputAction::WriteClose(ref element_type) => {
                    w.write_output_string(ecx, &format!("</{}>", &element_type));
                },

                &OutputAction::WriteVoid(ref element_type) => {
                    w.write_output_string(ecx, &format!("<{} />", &element_type));
                }
            }
        }
    }

    /*
    mod internal {
        use super::super::{OutputAction, WriteOutputActions, OutputActionWrite};
        use syntax::ext::base::ExtCtxt;
        use codegen::lang::{Lang, Html, Js};
        use codegen::output_string_writer::{WriteOutputStrings, OutputStringWrite};

        struct Wrapper<'s, 'cx, L: Lang + 's> {
            ecx: &'cx ExtCtxt<'cx>,
            w: &'s mut StringWrite<L>
        }

        macro_rules! lang {
            ($lang: ty) => {
                impl<'s, 'cx> OutputActionWrite for Wrapper<'s, 'cx, $lang> {
                    fn write_output_action(&mut self, output_action: &OutputAction) {
                        //&output_action.write_strings(&self.ecx, &mut self.w);
                    }
                }

                impl<S: WriteOutputActions> WriteOutputStrings<$lang> for S {
                    fn write_output_strings<'s, 'cx>(&self, ecx: &'cx ExtCtxt, w: &'s mut OutputStringWrite<$lang>) {
                        //let mut wrapper = Wrapper { ecx: ecx, w: w };
                        //&self.write_output_actions(&mut wrapper);
                    }
                }
            }
        }
        lang!(Html);
        //lang!(Js);
    }
    */
}

impl OutputActionWrite for Vec<OutputAction> {
    fn write_output_action(&mut self, output_action: &OutputAction) {
        self.push(output_action.clone());
    }
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

/*
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
*/

impl WriteJs for OutputAction {
    //fn write_js<W>(&self, js: &mut W) where W: JsWrite {
    fn write_js(&self, js: &mut JsWrite) {
        match *self {
            OutputAction::Write(ref contents) => {
                js.call_method("IncrementalDOM.text", &|pl| {
                    pl.param(&|ex| {
                        ex.string_lit(&contents);
                    });
                });
            },

            OutputAction::WriteResult(ref template_expr) => {
                js.call_method("IncrementalDOM.text", &|pl| {
                    pl.param(&|ex| {
                        template_expr.write_js_simple_expr(ex);
                    });
                });
            },

            OutputAction::WriteOpen(ref element_type) => {
                js.call_method("IncrementalDOM.elementOpen", &|pl| {
                    pl.param(&|ex| {
                        ex.string_lit(&element_type);
                    });
                });
            },

            OutputAction::WriteClose(ref element_type) => {
                js.call_method("IncrementalDOM.elementClose", &|pl| {
                    pl.param(&|ex| {
                        ex.string_lit(&element_type);
                    });
                });
            },

            OutputAction::WriteVoid(ref element_type) => {
                js.call_method("IncrementalDOM.elementVoid", &|pl| {
                    pl.param(&|ex| {
                        ex.string_lit(&element_type);
                    });
                });
            }
        }
    }
}

impl WriteJs for Vec<OutputAction> {
    fn write_js(&self, js: &mut JsWrite) {
        for output_action in self {
            output_action.write_js(js);
        }
    }
}

impl<S: WriteOutputActions> WriteJs for S {
    fn write_js(&self, js: &mut JsWrite) {
        let mut output_actions = Vec::new();
        self.write_output_actions(&mut output_actions);
        for output_action in &output_actions {
            output_action.write_js(js);
        }
    }
}
