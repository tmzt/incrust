
use std::fmt::Write;

use incrust_common::compiled_view::CompiledView;
use templates::render_template_main;
use models;


macro_rules! script_src {
    ($uri:expr) => (concat!["<script src=\"", $uri, "\"></script>"])
}

macro_rules! define_store_action {
    ($w: expr, $store: ident, action $name:ident => ($e:expr)) => (
        write!($w, "\t\tcase '{}': return {};\n",
            stringify!($name),
            format!("{{ {}: {} }}",
                stringify!($store),
                format!("(state.{})", stringify!($e))
            )
        )
    )
}

macro_rules! define_store {
    ($w: expr, store $store:ident {
        default => ($def:expr);
        $(action $act: ident => ($e:expr));*
    }) => ({
        let mut actions = String::new();
        $(define_store_action!(actions, $store, action $act => ($e));)*
        write!(actions, "\t\tdefault: return state;\n");

        let default_value = format!("\tstate = state || {{ {}: {} }};\n", stringify!($store), stringify!($def));
        let switch = format!("\tswitch (action.type) {{\n {}\n\t}};", actions);

        let body = format!("\n{}\n{}", default_value, switch);
        let create_store = format!("Redux.createStore(function store_handler_{}(state, action) {{ {} }})",
            stringify!($store), body);
        let store_fn = format!("\nfunction create_store_{}() {{return {}}}", stringify!($store), create_store);
        write!($w, "{}", store_fn);
    })
}

macro_rules! define_stores {
    ($(store $store:ident {
        default => ($def:expr);
        $(action $act: ident => ($e:expr));*
    });*) => ({
        let mut stores = String::new();
        $(define_store!(stores, store $store { default => ($def); $(action $act => ($e));* });)*

        stores
    })
}

pub fn render() -> String {
    let mut page = String::new();
    let mut main_js = String::new();

    // Render Rust and JS main template
        let main = render_template_main(&mut main_js);
        println!("Rendered main template: [{}]", main);

    // Define redux stores
        let store_js = define_stores!{
            store counter {
                default => (0);
                action INCREMENT => (counter + 1);
                action DECREMENT => (counter - 1)
            }
        };

    // TODO: Remove the 'start rendering' link
        let entry = r"
            document.addEventListener('DOMContentLoaded', function() {
                var root = document.querySelector('#root');
                function render(state) {
                    console.log('Patching IncrementalDOM');
                    IncrementalDOM.patch(root, render_view_root, state);
                }

                // Create a Redux store for our counter data
                var store_counter = create_store_counter();

                // Subscribe to updates
                store_counter.subscribe(function() {
                    render(store_counter.getState());
                });

                function start_counter() {
                    setInterval(function() {
                        store_counter.dispatch({type: 'INCREMENT'});
                    }, 1000);
                }

                document.querySelector('#actions .render').addEventListener('click', function() { start_counter(); });
            });";

    // Output HTML template
        write!(page, "<html><head>{}</head><body>{}</body></html>",
            format!("{}{}{}{}{}{}{}",
                "<title>Welcome to the incrust demo - rendering in isometric mode</title>",
                script_src!("/assets/js/incremental-dom-min.js"),
                script_src!("/assets/js/redux.js"),
                format!("<script>{}</script>", models::person_js()),
                format!("<script>{}</script>", main_js),
                format!("<script>{}</script>", store_js),
                format!("<script>{}</script>", entry)),

            format!("{}{}{}",
                format!("<div id=\"root\">{}</div>", main),
                format!("<br /><div id=\"js-code\"><code>{}</code></div>", main_js),
                format!("<br /><div id=\"actions\"><a class=\"render\" href=\"#\">start rendering</a></div>")));

    page
}
