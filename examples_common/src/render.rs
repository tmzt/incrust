
use std::fmt::Write;

use incrust_common::compiled_view::CompiledView;


macro_rules! script_src {
    ($uri:expr) => (concat!["<script src=\"", $uri, "\"></script>"])
}

pub fn render(main_fn: fn(html: &mut String, js: &mut String)) -> String {
    let mut page = String::new();
    let mut main_html = String::new();
    let mut main_js = String::new();

    // Render Rust and JS main template
        main_fn(&mut main_html, &mut main_js);
        println!("Rendered main template: [{}]", main_html);

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
            format!("{}{}{}{}{}{}",
                "<title>Welcome to the incrust demo - rendering in isometric mode</title>",
                script_src!("/assets/js/incremental-dom-min.js"),
                script_src!("/assets/js/redux.js"),
                format!("<script>{}</script>", main_js),
                format!("<script>{}</script>", store_js),
                format!("<script>{}</script>", entry)),

            format!("{}{}{}",
                format!("<div id=\"root\">{}</div>", main_html),
                format!("<br /><div id=\"js-code\"><code>{}</code></div>", main_js),
                format!("<br /><div id=\"actions\"><a class=\"render\" href=\"#\">start rendering</a></div>"))).unwrap();

    page
}
