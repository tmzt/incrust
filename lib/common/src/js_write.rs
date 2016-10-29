
use std::fmt::Write;

pub trait WriteJs {
    fn write_js(&self, js: &mut JsWrite);
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

pub trait JsWriteSimpleExpr {
    fn var_reference(&mut self, var_name: &str);
    fn string_lit(&mut self, lit: &str);

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

impl<T: Write> JsWriteSimpleExpr for T {
    fn var_reference(&mut self, var_name: &str) {
        write!(self, "{}", var_name);
    }

    fn string_lit(&mut self, lit: &str) {
        write!(self, "\"{}\"", lit);
    }

    fn open_paren(&mut self) {
        write!(self, "()");
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

#[test]
fn test_jsWrite_from_Write() {

}