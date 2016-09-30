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

use syntax::ext::base::IdentTT;
use syntax::parse::token;

mod template;
use template::expand_define_template;


#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(token::intern("define_template"),
            IdentTT(Box::new(expand_define_template), None, false));
}
