
use std::fmt::Write;


pub struct Person {
    first_name: String,
    last_name: String
}

impl Person {
    pub fn first_name(&self) -> &str { &self.first_name }
    pub fn last_name(&self) -> &str { &self.last_name }
}

macro_rules! data_struct_member {
    ($w: expr, $var: ident : $ty: ty) => ({
        write!($w, "\t{}: function() {{ return _d.{}; }},\n", stringify!($var), stringify!($var))
    })
}
macro_rules! data_struct {
    (struct $name:ident {
        $($var: ident : $ty: ty),*
    }) => ({
                let mut cls = String::new();
                let mut members = String::new();
                $(data_struct_member!(members, $var: $ty);)*

                let mut params = vec![$(concat!('"', stringify!($var), '"')),*];
                let mut d_members = String::new();
                $(write!(d_members, "\t{}: _args.shift(),\n", stringify!($var)).unwrap();)*
                write!(cls, "\nfunction {}() {{\n{}\n{}\nreturn Object.create({{\n {}}});\n}}", stringify!($name),
                    "var _args = [].slice.call(arguments);",
                    format!("var _d = {{\n{}}}", d_members),
                    members);
                cls
    })
}

pub fn person_js() -> String {
    data_struct! {
        struct Person {
            first_name: String,
            last_name: String
        }
    }
}
