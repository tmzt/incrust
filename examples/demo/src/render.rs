
use std::fmt::Write;

use incrust_common::compiled_view::CompiledView;
use templates::render_template_main;


macro_rules! script_src {
    ($uri:expr) => (concat!["<script src=\"", $uri, "\"></script>"])
}

pub fn render() -> String {
    let mut page = String::new();
    let mut main_js = String::new();

    // Render Rust and JS main template
        let main = render_template_main(&mut main_js);
        println!("Rendered main template: [{}]", main);

    // TODO: Remove the 'start rendering' link
        let entry = r"
            document.addEventListener('DOMContentLoaded', function() {
                var root = document.querySelector('#root');
                var data = {};
                function startRendering() {
                    setInterval(function() {
                        console.log('Patching IncrementalDOM');
                        IncrementalDOM.patch(root, render_view_root, data);
                    }, 1000);
                };

                document.querySelector('#actions .render').addEventListener('click', function() { startRendering(); });
            });";

    // Output HTML template
        write!(page, "<html><head>{}</head><body>{}</body></html>",
            format!("{}{}{}",
                "<title>Welcome to the incrust demo - rendering in isometric mode</title>",
                script_src!("/assets/js/incremental-dom.js"),
                format!("<script>{}</script>", entry)),

            format!("{}{}{}",
                format!("<div id=\"root\">{}</div>", main),
                format!("<br /><div id=\"js-code\"><code>{}</code></div>", main_js),
                format!("<br /><div id=\"actions\"><a class=\"render\" href=\"#\">start rendering</a></div>")));

    page
}
