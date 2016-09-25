
use std::fmt::Write;

/*
macro_rules! js_args {
    ($w:expr, ) => (());
    ($args:tt) => (concat!["(", $args, ");"]);
}

macro_rules! js_call {
    ($func_name:ident, $($args:tt)*) => (concat![stringify![$func_name], $($args)*]);
}

macro_rules! do_jsr_call {
    ($func_name:ident, $element_type:ident, @single $($args:tt)*) => { concat!(stringify!($func_name), $(stringify!($args))*) };
    ($func_name:ident, $element_type:ident, @count $($rest:expr),*) => { ... };
}
*/

macro_rules! script_src {
    ($uri:expr) => (concat!["<script src=\"", $uri, "\"></script>"])
}

macro_rules! jsr_call {
    // text node
    (text, $text:tt) => {
        concat!("IncrementalDOM.text('", $text, "');")
    };

    // element without attributes
    ($func_name:ident, $element_type:ident) => {
        concat!("IncrementalDOM.", stringify!($func_name), "('", stringify!($element_type), "');") };

    // element with attributes
    ($func_name:ident, $element_type:ident, $($key:ident => $value:expr,)+) => {
        concat!("IncrementalDOM.", stringify!($func_name), "('", stringify!($element_type), "', ",
            "[", $("'", stringify!($key), "', ", stringify!($value), ", "),*, "]); ") };
    ($func_name:ident, $element_type:ident, $($key:ident => $value:expr),*) => {
        concat!("IncrementalDOM.", stringify!($func_name), "('", stringify!($element_type), "', ",
            "[", $("'", stringify!($key), "', ", stringify!($value), ", "),*, "]); ") };
}

macro_rules! response {
    () => ("");
}

macro_rules! output_rust_call {
    ($w:expr, elementOpen $element_type:ident) => { write!($w, "<{}>", stringify!($element_type)); };
    ($w:expr, elementClose $element_type:ident) => { write!($w, "<{}>", stringify!($element_type)); };
    ($w:expr, text $text:tt) => { write!($w, "{}", $text); };
}

macro_rules! output_js_call {
    ($w:expr, elementOpen $element_type:ident) => { write!($w, "{}", jsr_call!(elementOpen, $element_type)); };
    ($w:expr, elementClose $element_type:ident) => { write!($w, "{}", jsr_call!(elementClose, $element_type)); };
    ($w:expr, text $text:tt) => { write!($w, "{}", jsr_call!(text, $text)); };

    ($w:expr, render_on_load $template_name:ident) => {
        write!($w, "document.addEventListener('DOMContentLoaded', function() {{ IncrementalDOM.patch(document.querySelector('#{}'), template_{}); }});",
            stringify!($template_name), stringify!($template_name));
    };
}

macro_rules! incr_template_gen {
    ($js:expr, $rs:expr, view $template_name:ident [ $($inner:tt)* ] $($rest:tt)*) => {
        // Output JS function and Rust code
        write!($js, "function template_{}() {{", stringify!($template_name));
            incr_template_gen!($js, $rs, $($inner)*);
        write!($js, "}};")
    };

    ($js:expr, $rs:expr) => ("");
    ($js:expr, $rs:expr, $text:tt) => (
        output_js_call!($js, text $text);
        output_rust_call!($rs, text $text);
    );

    ($js:expr, $rs:expr, $element_type:ident [ $($inner:tt)* ] $($rest:tt)*) => {
        output_js_call!($js, elementOpen $element_type);
        output_rust_call!($rs, elementOpen $element_type);

        //write!($rs, "{}", emit_rust_template!($element_type, $element_type, $element_type));
        //emit_rust_template!($element_type, $element_type, $element_type);

        incr_template_gen!($js, $rs, $($inner)*);

        output_js_call!($js, elementClose $element_type);
        output_rust_call!($rs, elementClose $element_type);
    };
}

macro_rules! incr_template {
    (view $template_name:ident [ ]) => { concat!["function template_", stringify!($template_name), "(){};"] };
    (view $template_name:ident [ $($inner:tt)* ] $($rest:tt)*) => { concat!["function template_", stringify!($template_name), "(){", incr_template!($($inner)*), "};"] };

    () => ("");
    ($e:tt) => ( concat!["IncrementalDOM.text('", $e, "');"] );

    ($element_type:ident [ $($inner:tt)* ] $($rest:tt)*) => { concat![
        jsr_call!(elementOpen, $element_type),
            incr_template!($($inner)*),
        jsr_call!(elementClose, $element_type),
        incr_template!($($rest)*)
    ]};
}

fn render_template() -> String {
    emit_rust_template!(view root [ p [ ] ])
}

fn render_page(page: &mut String) {
    let mut js = String::new();
    let mut rs = String::new();

    // Populate js and rs variables
        incr_template_gen!(&mut js, &mut rs, view root [ p [ "test" ] ]);
        output_js_call!(&mut js, render_on_load root);

    // HTML template
    write!(page, "<html><head><title>incrust demo</title>");
    write!(page, script_src!("/assets/js/incremental-dom.js"));
    write!(page, "<script>{}</script>", js);

    let s = render_template();
    write!(page, "</head><body><div id=\"root\">{}</div><div id=\"rust-code\">{}</div></body></html>", s, rs);
}

pub fn render() -> String {
    let mut page = String::new();

    render_page(&mut page);

    page
}

/*
pub fn render() -> String {
    concat![
        "<html><head><title>incrust demo</title>",
        script_src!("/assets/js/incremental-dom.js"),
        "<script>",
            incr_template![
                view root [ p [ "test" ] ]
            ],
            "document.addEventListener('DOMContentLoaded', function() { IncrementalDOM.patch(document.querySelector('#root'), template_root); });",
        "</script>",
        "</head><body><div id=\"root\"></div></body></html>"
    ]

//    jsr_call! [elementOpen, p, color => "red", ]
//    jsr_call! [elementOpen, p, color => "red"]
}
*/