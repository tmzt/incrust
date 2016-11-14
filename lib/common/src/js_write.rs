
use std::fmt::Write;
use common_write::WriteAs;
use value::Value;
use object_expr::ObjectExprAssignment;


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

/// Represents the state of the JS output where we are in a switch expression body.
pub trait WriteJsSwitchBody {
    fn write_js_switch_body(&self, switch: &mut JsWriteSwitchBody);
}

pub trait JsWrite {
    fn function(&mut self, func_name: &str, f: &Fn(&mut JsWrite));

    fn let_statement(&mut self, var_name: &str, f: &FnOnce(&mut JsWriteSimpleExpr));
    fn call_method(&mut self, method_name: &str, f: &Fn(&mut JsWriteParamList));
    //fn write_simple_expr<F>(&mut self, f: F) where F: FnOnce(&mut JsWriteSimpleExpr);

    /// Switch expression where the value to match is a simple variable reference
    fn switch_expr_simple(&mut self, var_name: &str, f: &Fn(&mut JsWriteSwitchBody));
}

pub trait JsWriteFunctions {
    fn function(&mut self, func_name: &str, args: Vec<&str>, f: &Fn(&mut JsWrite));
}

pub trait JsWriteExpr {
    fn write_value(&mut self, value: &Value);
}

pub trait JsWriteSimpleExpr {
    fn var_reference(&mut self, var_name: &str);
    fn string_lit(&mut self, lit: &str);
    fn int64_lit(&mut self, n: i64);
    fn int32_lit(&mut self, n: i32);

    fn open_brace(&mut self);
    fn close_brace(&mut self);

    fn open_paren(&mut self);
    fn close_paren(&mut self);

    fn binop_plus(&mut self);
    fn binop_minus(&mut self);
}

pub trait JsWriteObjectExpr {
    fn write_object_expr(&mut self, cls_name: &str, f: &Fn(&mut JsWriteObjectExprBody));
}

pub trait JsWriteObjectExprBody {
    fn write_field_assignment(&mut self, field_name: &str, value: Value);
}

impl<T: Write> JsWriteObjectExpr for T {
    fn write_object_expr(&mut self, cls_name: &str, f: &Fn(&mut JsWriteObjectExprBody)) {
        write!(self, "{} {{", &cls_name);
        f(self);
        write!(self, "}}");
    }
}

impl<T: Write> JsWriteObjectExprBody for T {
    fn write_field_assignment(&mut self, field_name: &str, value: Value) {
    }
}

/// Allow writing switch case labels in a simplified expression syntax. This supports the Redux use case.
pub trait JsWriteSwitchBody {
    fn case_str(&mut self, case_str: &str, f: &Fn(&mut JsWriteExpr));
    fn default_case(&mut self, f: &Fn(&mut JsWriteExpr));
}

pub trait JsWriteFuncParamList {
    fn param(&mut self, var_name: &str);
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

    fn switch_expr_simple(&mut self, var_name: &str, f: &Fn(&mut JsWriteSwitchBody)) {
        write!(self, "switch ({}) {{", &var_name);
        f(self);
        write!(self, "}};");
    }
}

impl<T: Write> JsWriteSwitchBody for T {
    fn case_str(&mut self, case_str: &str, f: &Fn(&mut JsWriteExpr)) {
        write!(self, "case '{}': return ", &case_str);
        f(self);
        write!(self, ";");
    }

    fn default_case(&mut self, f: &Fn(&mut JsWriteExpr)) {
        write!(self, "default: return ");
        f(self);
        write!(self, ";");
    }
}

impl<T: Write> JsWriteFunctions for T {
    fn function(&mut self, func_name: &str, args: Vec<&str>, f: &Fn(&mut JsWrite)) {
        let args_str = &args.join(", ");
        write!(self, "function {}({}) {{ ", func_name, &args_str);
        f(self);
        write!(self, "}};");
    }
}

impl<T: Write> JsWriteExpr for T {
    fn write_value(&mut self, value: &Value) {
        match value {
            &Value::SimpleExprValue(ref simple_expr) => {
                simple_expr.write_js_simple_expr(self);
            },
            
            &Value::ObjectExprValue(ref object_expr) => {
                write!(self, "{{");
                for assignment in object_expr.assignments() {
                    let (name, simple_expr) = match assignment {
                        &ObjectExprAssignment::NewSimpleExprValue(ref name, ref simple_expr) => (name, simple_expr),
                        &ObjectExprAssignment::UpdateSimpleExprValue(ref name, ref simple_expr) => (name, simple_expr)
                    };

                    write!(self, "{}: ", name).unwrap();
                    simple_expr.write_js_simple_expr(self);
                    write!(self, ", ").unwrap();
                }
                write!(self, "}}");
            }
        }
    }
}

impl<T: Write> JsWriteSimpleExpr for T {
    fn var_reference(&mut self, var_name: &str) {
        write!(self, "{}", var_name);
    }

    fn string_lit(&mut self, lit: &str) {
        write!(self, "\"{}\"", lit);
    }

    fn int32_lit(&mut self, n: i32) {
        write!(self, "{}", n);
    }

    fn int64_lit(&mut self, n: i64) {
        write!(self, "{}", n);
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