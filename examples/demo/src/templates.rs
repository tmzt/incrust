// TODO: Look into expanding these macros
use std::fmt::Write;

pub fn render_template_main(html: &mut String, js: &mut String) {
    define_template! main {
        view root [
            div [
                h1 [ {"Counter: "} {"  "} {(data.counter)} ]
            ]
        ]
    }

    emit_js_view_main!(js);
    emit_rust_view_main!(html);
}
