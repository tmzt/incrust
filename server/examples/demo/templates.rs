// TODO: Look into expanding these macros
use std::fmt::Write;

template! main {
    view root [
        div [
            h1 [ {"Counter: "} {counter} ]
        ]
    ]

    store counter {
        default => (0);
        action INCREMENT => (counter + 1);
        action DECREMENT => (counter - 1)
    }

}

pub fn render_template_main(html: &mut String, js: &mut String, head_tags: &mut String) {
    render_template_root!(html, js, main);

    write!(head_tags, "<script>{},\n\t{}</script>\n",
        "register_main_view(function() { return create_store_counter(); }",
        "function(store) { setInterval(function() { store.dispatch({type: 'INCREMENT'}); }, 1000); });");
}
