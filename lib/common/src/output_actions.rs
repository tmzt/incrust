
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
    use syntax::codemap::{DUMMY_SP, Span};
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
            ecx.span_warn(DUMMY_SP, &format!("Writing output action: {:?}", &self));
            match self {
                &OutputAction::Write(ref contents) => {
                    ecx.span_warn(DUMMY_SP, &format!("Writing output string for Write output action: {}", contents));
                    w.write_output_string(ecx, &contents);
                },

                &OutputAction::WriteResult(ref simple_expr) => {
                    ecx.span_warn(DUMMY_SP, &format!("Writing output string for WriteResult"));
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
}

impl OutputActionWrite for Vec<OutputAction> {
    fn write_output_action(&mut self, output_action: &OutputAction) {
        self.push(output_action.clone());
    }
}

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
