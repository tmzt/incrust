// TODO: Look into expanding these macros
use std::fmt::Write;

template! main {
    store counter {
        default => (0);
        action INCREMENT => (counter + 1);
        action DECREMENT => (counter - 1)
    }

    view root [
        div [
            h1 [ {"Counter: "}{counter} ]
        ]
    ]
}

pub fn render_template_main(html: &mut String, js: &mut String, head_tags: &mut String) {
    //render_template_root!(html, js, main, Html);
    //render_template_root!(html, js, main, Js);

    render_output!(html, js, main, view, root, Html);
    render_output!(html, js, main, view, root, Js);
    render_output!(html, js, main, store, counter, Js);

    write!(head_tags, "<script>{}{}{}{}</script>\n",
        "function store_factory() {
            return Redux.createStore(function(store, action) { return rusttemplate_store_template_main_counter(store, action); });
        };\r\n",
        "function view_factory() {
            return rusttemplate_render_template_main_view_root_calls;
        };\r\n",
        "function start(store) {
            setInterval(function() { store.dispatch({type: 'INCREMENT'}); }, 1000);
        };\r\n",
        "register_main_view(store_factory, view_factory, start);\r\n");

}
