
use std::fmt::Write;

pub trait WriteJs {
    fn write_js<W>(&self, js: &mut W) where W: JsWrite;
}

pub trait JsWrite {
    fn open_function(&mut self, func_name: &str);
    fn close_function(&mut self);

    fn let_statement<F>(&mut self, var_name: &str, f: F) where F: FnOnce(&mut JsWriteSimpleExpr);
    fn call_method<F>(&mut self, method_name: &str, f: F) where F: FnOnce(&mut JsWriteParamList);
    fn write_simple_expr<F>(&mut self, f: F) where F: FnOnce(&mut JsWriteSimpleExpr);
}

pub trait JsWriteSimpleExpr {
    fn var_reference(&mut self, var_name: &str);
    fn binop_add(&mut self);
    fn binop_minus(&mut self);
}

pub trait JsWriteParamList {
    fn param<F>(&mut self, f: F) where F: FnOnce(&JsWriteSimpleExpr);
}

impl<T: Write> JsWrite for T {
    fn open_function(&mut self, func_name: &str) {
        write!(self, "function {}() {{", func_name);
    }

    fn close_function(&mut self) {
        write!(self, "}};");
    }

    fn let_statement<F>(&mut self, var_name: &str, f: F) where F: FnOnce(&mut JsWriteSimpleExpr) {
    }

    fn call_method<F>(&mut self, method_name: &str, f: F) where F: FnOnce(&mut JsWriteParamList) {
    }

    fn write_simple_expr<F>(&mut self, f: F) where F: FnOnce(&mut JsWriteSimpleExpr) {
    }
}

impl<T: Write> JsWriteSimpleExpr for T {
    fn var_reference(&mut self, var_name: &str) {
        write!(self, "{}", var_name);
    }

    fn binop_add(&mut self) {
        write!(self, " + ");
    }

    fn binop_minus(&mut self) {
        write!(self, " - ");
    }
}

impl<T: Write> JsWriteParamList for T {
    fn param<F>(&mut self, f: F) where F: FnOnce(&JsWriteSimpleExpr) {
        //f(&self);
    }
}

#[test]
fn test_jsWrite_from_Write() {

}