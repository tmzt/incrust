// TODO: Look into expanding these macros
use std::fmt::Write;

pub fn render_template_main(js: &mut String) -> String {
    define_template! main {
        view root [
            div [
                h1 [ {"Counter: "} {"  "} {(data.counter)} ]
            ]
        ]
    }

    let mut out = String::new();
    emit_js_view_main!(js);
    emit_rust_view_main!(out);
    out
}
