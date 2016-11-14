


macro_rules! render_example_page {
    ($page: expr, $template_name: ident, $view_name: ident, $store_name: ident) => ({
        macro_rules! script_src (($uri:expr) => (concat!["<script src=\"", $uri, "\"></script>"]));

        let mut main_html = String::new();
        let mut main_js = String::new();
        let mut head_tags = String::new();

        // TODO: Remove the 'start rendering' link
        let entry = r"
                document.addEventListener('DOMContentLoaded', function() {
                    function start(store) {
                        setInterval(function() { store.dispatch({type: 'INCREMENT'}); }, 1000);
                    };

                    var view = view_factory();
                    var root = document.querySelector('#root');
                    function render(state) {
                        console.log('Patching IncrementalDOM');
                        IncrementalDOM.patch(root, view, state);
                    }

                    var store = store_factory();

                    // Subscribe to updates
                    store.subscribe(function() {
                        render(store.getState());
                    });

                    document.querySelector('#actions .render').addEventListener('click', function() { start(store); });
                });
        }";

        write!(head_tags, r"
            <script>
            (function(){{
                {}
                {}
                {}
            }}();
            </script>",
            format!("function store_factory() {{ return Redux.createStore(rusttemplate_store_template_{}_{}; }};", stringify!($template_name), stringify!($view_name)),
            format!("function view_factory() {{ return rusttemplate_render_template_{}_view_{}_calls; }};", stringify!($template_name), stringify!($view_name)),
            entry
        ).unwrap();

        // Render Rust and JS main template
        render_output!(main_html, main_js, $template_name, view, $view_name, Html);
        /*
        render_output!(&mut main_html, &mut main_js, $template_name, view, $view_name, Js);
        render_output!(&mut main_html, &mut main_js, $template_name, store, $store_name, Js);
        println!("Rendered main template: [{}]", &main_html);
        */

        // Output HTML template
        write!(page, "<html><head>{}</head><body>{}</body></html>",
            format!("{}{}{}{}{}",
                "<title>Welcome to the incrust demo</title>",
                script_src!("/assets/js/incremental-dom-min.js"),
                script_src!("/assets/js/redux.js"),
                format!("<script>{}</script>", &main_js),
                head_tags),

            format!("{}{}{}",
                format!("<div id=\"root\">{}</div>", &main_html),
                format!("<br /><div id=\"js-code\"><code>{}</code></div>", &main_js),
                format!("<br /><div id=\"actions\"><a class=\"render\" href=\"#\">start rendering</a></div>"))).unwrap();

        page
    })
}


#[macro_export]
macro_rules! example {
    ($template_name: ident, $view_name: ident, $store_name: ident, $($extra_js: expr),*) => (
        use std::path::Path;
        use std::fmt::Write;
        use nickel::{ Nickel, HttpRouter, StaticFilesHandler };

        fn statics() -> StaticFilesHandler {
            const CARGO_MANIFEST_DIR: Option<&'static str> = option_env!("CARGO_MANIFEST_DIR");
            let root_dir = CARGO_MANIFEST_DIR.map(|s| Path::new(s))
                .expect("Must provide CARGO_MANIFEST_DIR as cargo does pointing to the folder containing public/");
            StaticFilesHandler::new(root_dir.join("public"))
        }

        fn render() -> String {
            let mut page = String::new();
            macro_rules! script_src (($uri:expr) => (concat!["<script src=\"", $uri, "\"></script>"]));

            let mut main_html = String::new();
            let mut main_js = String::new();
            let mut head_tags = String::new();

            // TODO: Remove the 'start rendering' link
            let entry = r"
                    document.addEventListener('DOMContentLoaded', function() {
                        var view = view_factory();
                        var root = document.querySelector('#root');
                        function render(state) {
                            console.log('Patching IncrementalDOM');
                            IncrementalDOM.patch(root, view, state);
                        }

                        var store = store_factory();

                        // Subscribe to updates
                        store.subscribe(function() {
                            render(store.getState());
                        });

                        document.querySelector('#actions .render').addEventListener('click', function() { start(store); });
                    });
            ";

            let mut extra_js = String::new();
            $(
                writeln!(&mut extra_js, "{}", $extra_js);
            )*

            write!(head_tags, r"
                <script>
                (function(){{
                    {}
                    {}
                    {}
                    {}
                }})();
                </script>",
                &extra_js,
                format!("function store_factory() {{ return Redux.createStore(rusttemplate_store_template_{}_{}); }};", stringify!($template_name), stringify!($store_name)),
                format!("function view_factory() {{ return rusttemplate_render_template_{}_view_{}_calls; }};", stringify!($template_name), stringify!($view_name)),
                entry
            ).unwrap();

            // Render Rust and JS main template
            render_output!(&mut main_html, &mut main_js, $template_name, view, $view_name, Html);
            render_output!(&mut main_html, &mut main_js, $template_name, view, $view_name, Js);
            render_output!(&mut main_html, &mut main_js, $template_name, store, $store_name, Js);
            println!("Rendered main template: [{}]", &main_html);

            // Output HTML template
            write!(&mut page, "<html><head>{}</head><body>{}</body></html>",
                format!("{}{}{}{}{}",
                    "<title>Welcome to the incrust demo</title>",
                    script_src!("/assets/js/incremental-dom-min.js"),
                    script_src!("/assets/js/redux.js"),
                    format!("<script>{}</script>", &main_js),
                    head_tags),

                format!("{}{}{}",
                    format!("<div id=\"root\">{}</div>", &main_html),
                    format!("<br /><div id=\"js-code\"><code>{}</code></div>", &main_js),
                    format!("<br /><div id=\"actions\"><a class=\"render\" href=\"#\">start rendering</a></div>"))).unwrap();

            page
        }

        fn main() {
            let mut server = Nickel::new();
            server.utilize(statics());
            server.get("**", middleware!(render()));

            server.listen("127.0.0.1:6767").unwrap();
        }
    )
}

/*
#[macro_export]
macro_rules! render_example {
    ($template_name: ident, $view_name: ident, $store_name: ident) => ({
        macro_rules! script_src (($uri:expr) => (concat!["<script src=\"", $uri, "\"></script>"]));
        use std::fmt::Write;

        let mut page = String::new();
        let mut main_html = String::new();
        let mut main_js = String::new();
        let mut head_tags = String::new();

        // TODO: Remove the 'start rendering' link
        let entry = r"
                document.addEventListener('DOMContentLoaded', function() {
                    function start(store) {
                        setInterval(function() { store.dispatch({type: 'INCREMENT'}); }, 1000);
                    };

                    var view = view_factory();
                    var root = document.querySelector('#root');
                    function render(state) {
                        console.log('Patching IncrementalDOM');
                        IncrementalDOM.patch(root, view, state);
                    }

                    var store = store_factory();

                    // Subscribe to updates
                    store.subscribe(function() {
                        render(store.getState());
                    });

                    document.querySelector('#actions .render').addEventListener('click', function() { start(store); });
                });
        }";

        write!(head_tags, r"
            <script>
            (function(){{
                {}
                {}
                {}
            }}();
            </script>",
            format!("function store_factory() {{ return Redux.createStore(rusttemplate_store_template_{}_{}; }};", stringify!($template_name), stringify!($view_name)),
            format!("function view_factory() {{ return rusttemplate_render_template_{}_view_{}_calls; }};", stringify!($template_name), stringify!($view_name)),
            entry
        ).unwrap();

        // Render Rust and JS main template
        render_output!(main_html, main_js, $template_name, view, $view_name, Html);
        /*
        render_output!(&mut main_html, &mut main_js, $template_name, view, $view_name, Js);
        render_output!(&mut main_html, &mut main_js, $template_name, store, $store_name, Js);
        println!("Rendered main template: [{}]", &main_html);

        // Output HTML template
        format!("<html><head>{}</head><body>{}</body></html>",
            format!("{}{}{}{}{}",
                "<title>Welcome to the incrust demo</title>",
                script_src!("/assets/js/incremental-dom-min.js"),
                script_src!("/assets/js/redux.js"),
                format!("<script>{}</script>", &main_js),
                head_tags),

            format!("{}{}{}",
                format!("<div id=\"root\">{}</div>", &main_html),
                format!("<br /><div id=\"js-code\"><code>{}</code></div>", &main_js),
                format!("<br /><div id=\"actions\"><a class=\"render\" href=\"#\">start rendering</a></div>")))
        */

        page
    })
}
*/

/*
pub fn render(main_fn: fn(html: &mut String, js: &mut String, head_tags: &mut String)) -> String {

    let mut page = String::new();
    let mut main_html = String::new();
    let mut main_js = String::new();
    let mut head_tags = String::new();

    // Render Rust and JS main template
        main_fn(&mut main_html, &mut main_js, &mut head_tags);
        println!("Rendered main template: [{}]", main_html);

    // TODO: Remove the 'start rendering' link
        entry_script!(head_tags, main, root);

    // Output HTML template
        write!(page, "<html><head>{}</head><body>{}</body></html>",
            format!("{}{}{}{}{}",
                "<title>Welcome to the incrust demo</title>",
                script_src!("/assets/js/incremental-dom-min.js"),
                script_src!("/assets/js/redux.js"),
                format!("<script>{}</script>", main_js),
                head_tags),

            format!("{}{}{}",
                format!("<div id=\"root\">{}</div>", main_html),
                format!("<br /><div id=\"js-code\"><code>{}</code></div>", main_js),
                format!("<br /><div id=\"actions\"><a class=\"render\" href=\"#\">start rendering</a></div>"))).unwrap();

    page
}


#[macro_export]
macro_rules! main_template (
    ($template_name: ident, $view_name: ident, $store_name: ident) => ({
        fn render_template_main(html: &mut String, js: &mut String, head_tags: &mut String) {
            render_template_items!(&mut html, &mut js, main, root);
        }
    })
);
*/
