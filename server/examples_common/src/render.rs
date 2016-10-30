
use std::fmt::Write;


macro_rules! script_src {
    ($uri:expr) => (concat!["<script src=\"", $uri, "\"></script>"])
}

pub fn render(main_fn: fn(html: &mut String, js: &mut String, head_tags: &mut String)) -> String {
    let mut page = String::new();
    let mut main_html = String::new();
    let mut main_js = String::new();
    let mut head_tags = String::new();

    // Render Rust and JS main template
        main_fn(&mut main_html, &mut main_js, &mut head_tags);
        println!("Rendered main template: [{}]", main_html);

    // TODO: Remove the 'start rendering' link
        let entry = r"
            function register_main_view(store_factory, start) {
                document.addEventListener('DOMContentLoaded', function() {
                    var root = document.querySelector('#root');
                    function render(state) {
                        console.log('Patching IncrementalDOM');
                        IncrementalDOM.patch(root, render_view_root, state);
                    }

                    var store = store_factory();

                    // Subscribe to updates
                    store.subscribe(function() {
                        render(store.getState());
                    });

                    document.querySelector('#actions .render').addEventListener('click', function() { start(store); });
                });
            }";

    // Output HTML template
        write!(page, "<html><head>{}</head><body>{}</body></html>",
            format!("{}{}{}{}{}{}",
                "<title>Welcome to the incrust demo - rendering in isometric mode</title>",
                script_src!("/assets/js/incremental-dom-min.js"),
                script_src!("/assets/js/redux.js"),
                format!("<script>{}</script>", main_js),
                format!("<script>{}</script>", entry),
                head_tags),

            format!("{}{}{}",
                format!("<div id=\"root\">{}</div>", main_html),
                format!("<br /><div id=\"js-code\"><code>{}</code></div>", main_js),
                format!("<br /><div id=\"actions\"><a class=\"render\" href=\"#\">start rendering</a></div>"))).unwrap();

    page
}
