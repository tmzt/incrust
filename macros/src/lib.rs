
/*
macro_rules! emit_rust_compiled_template {
    ($compiled_view:expr) => ({
        extern crate incrust_common;
        use incrust_common::codegen::create_template_block;
        create_template_block();
        use incrust_common::{Template, tts_to_template};
        Template::from_views(vec![])
    })
}
*/


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
