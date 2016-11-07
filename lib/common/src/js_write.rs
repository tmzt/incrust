
use std::fmt::Write;

pub trait WriteJs {
    fn write_js(&self, js: &mut JsWrite);
}

/// Request the object write itself out as a series of named Javascript functions
pub trait WriteJsFunctions {
    fn write_js_functions(&self, js: &mut JsWriteFunctions);
}

/// Implicit implementation of WriteJsFunctions for a Vec<&WriteJsFunctions>.
/// Simply writes out the functions for each element in the vector.
impl <T: WriteJsFunctions> WriteJsFunctions for Vec<T> {
    fn write_js_functions(&self, js: &mut JsWriteFunctions) {
        for element in self {
            element.write_js_functions(js);
        }
    }
}

pub trait WriteJsSimpleExpr {
    fn write_js_simple_expr(&self, js: &mut JsWriteSimpleExpr);
}

pub trait JsWrite {
    fn function(&mut self, func_name: &str, f: &Fn(&mut JsWrite));

    fn let_statement(&mut self, var_name: &str, f: &FnOnce(&mut JsWriteSimpleExpr));
    fn call_method(&mut self, method_name: &str, f: &Fn(&mut JsWriteParamList));
    //fn write_simple_expr<F>(&mut self, f: F) where F: FnOnce(&mut JsWriteSimpleExpr);
}

pub trait JsWriteFunctions {
    fn function(&mut self, func_name: &str, f: &Fn(&mut JsWrite));    
}

pub trait JsWriteSimpleExpr {
    fn var_reference(&mut self, var_name: &str);
    fn string_lit(&mut self, lit: &str);

    fn open_brace(&mut self);
    fn close_brace(&mut self);

    fn open_paren(&mut self);
    fn close_paren(&mut self);

    fn binop_plus(&mut self);
    fn binop_minus(&mut self);
}

pub trait JsWriteParamList {
    fn param(&mut self, f: &Fn(&mut JsWriteSimpleExpr));
}

impl<T: Write> JsWrite for T {
    fn let_statement(&mut self, var_name: &str, f: &FnOnce(&mut JsWriteSimpleExpr)) {
    }

    fn function(&mut self, func_name: &str, f: &Fn(&mut JsWrite)) {
        write!(self, "function {}() {{", func_name);
        f(self);
        write!(self, "}};");
    }

    fn call_method(&mut self, method_name: &str, f: &Fn(&mut  JsWriteParamList)) {
        write!(self, "{}(", method_name);
        f(self);
        write!(self, ");\r\n");
    }

    /*
    fn write_simple_expr<F>(&mut self, f: F) where F: FnOnce(&mut JsWriteSimpleExpr) {
    }
    */
}

impl<T: Write> JsWriteFunctions for T {
    fn function(&mut self, func_name: &str, f: &Fn(&mut JsWrite)) {
        write!(self, "function {}() {{", func_name);
        f(self);
        write!(self, "}};");
    }
}

impl<T: Write> JsWriteSimpleExpr for T {
    fn var_reference(&mut self, var_name: &str) {
        write!(self, "{}", var_name);
    }

    fn string_lit(&mut self, lit: &str) {
        write!(self, "\"{}\"", lit);
    }

    fn open_brace(&mut self) {
        write!(self, "{{");
    }

    fn close_brace(&mut self) {
        write!(self, "}}");
    }

    fn open_paren(&mut self) {
        write!(self, "(");
    }

    fn close_paren(&mut self) {
        write!(self, ")");
    }

    fn binop_plus(&mut self) {
        write!(self, " + ");
    }

    fn binop_minus(&mut self) {
        write!(self, " - ");
    }
}

impl<T: Write> JsWriteParamList for T {
    fn param(&mut self, f: &Fn(&mut JsWriteSimpleExpr)) {
        f(self);
    }
}

mod output_strings {
    use super::{WriteJs, WriteJsFunctions};
    use std::iter::Iterator;
    use syntax::codemap::{Span, DUMMY_SP};
    use syntax::ext::base::ExtCtxt;
    use syntax::ast;
    use codegen::lang::{Lang, Html, Js};
    use codegen::output_string_writer::{WriteOutputStrings, OutputStringWrite};

    /*
    impl<S: WriteJs> WriteOutputStrings<Js> for S {
        fn write_output_strings<'s, 'cx>(&self, ecx: &'cx ExtCtxt, w: &'s mut OutputStringWrite<Js>) {
            let mut out = String::new();
            self.write_js(&mut out);
            w.write_output_string(ecx, &mut out);
        }
    }
    */

    impl<S: WriteJsFunctions> WriteOutputStrings<Js> for S {
        fn write_output_strings<'s, 'cx>(&self, ecx: &'cx ExtCtxt, w: &'s mut OutputStringWrite<Js>) {
            ecx.span_warn(DUMMY_SP, &format!("Writing output strings for js functions"));

            let mut out = String::new();
            self.write_js_functions(&mut out);
            w.write_output_string(ecx, &mut out);
        }
    }
}

#[test]
fn test_jsWrite_from_Write() {

}