use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult};

pub trait IntoJsOutputCall {
    fn into_js_output_call(&self) -> String;
}

pub trait IntoJsFunction {
    fn into_js_function<'cx>(&self, ecx: &'cx ExtCtxt) -> String;
}