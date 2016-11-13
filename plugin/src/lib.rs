#![crate_name="incrust_plugin"]
#![crate_type="dylib"]
#![feature(quote, plugin_registrar, rustc_private)]

#[macro_use]
extern crate log;

extern crate syntax;
extern crate rustc;
extern crate rustc_plugin;

extern crate itertools;
extern crate incrust_common;

use rustc_plugin::Registry;

use syntax::ext::base::{NormalTT, IdentTT};
use syntax::parse::token;

mod template_syntax;
mod actions;
mod expr;


#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(token::intern("template"),
            IdentTT(Box::new(template_syntax::expander::expand_template), None, false));

    reg.register_syntax_extension(token::intern("render_output"),
            NormalTT(Box::new(template_syntax::expander::expand_render_output), None, false));

    // Store definitions
    actions::register_store(reg);
//    reg.register_syntax_extension(token::intern("define_store"),
//            IdentTT(Box::new(actions::expand_define_store), None, false));
}
