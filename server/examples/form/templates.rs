// TODO: Look into expanding these macros
use std::fmt::Write;

use models::person_js;

pub fn render_template_main(html: &mut String, js: &mut String, head_tags: &mut String) {
    define_template! main {
        view root [
            p [ {"First name:  "} {(data.first_name)} ]
            p [ {"Last name:  "} {(data.last_name)} ]
            div [
                form [
                    input []
                    input []
                ]
            ]
        ]
    }

    // Define models
    write!(head_tags, "\n<script>{}</script>\n", person_js());

    // Define redux stores
    write!(head_tags, "\n<script>{}</script>\n", define_stores!{
        store person {
            default => ({});
            action SET_FIRST_NAME => (Person {first_name: "first_name"});
            action SET_LAST_NAME => (Person {last_name: "last_name"})
        }
    });

    write!(head_tags, "\n<script>{},\n\t{}</script>\n",
        "register_main_view(function() { return create_store_person(); }",
        "function(store) { setInterval(function() { store.dispatch({type: 'SET_FIRST_NAME'}); }, 1000); });");

    emit_js_view_main!(js);
    emit_rust_view_main!(html);
}
