
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
macro_rules! data_struct_d_var {
    ($w: expr, $var: ident : $ty: ty) => ({
        write!($w, "\n\t\t{}: _args.shift(),", stringify!($var))
    })
}
macro_rules! data_struct_ctor_param {
    ($w: expr, $param: ident) => ({write!($w, "{}: _args[]")})
}
macro_rules! data_struct {
    (struct $name:ident {
        $($var: ident : $ty: ty),*
    }) => ({
                let mut cls = String::new();
                let mut members = String::new();
                $(data_struct_member!(members, $var: $ty);)*

                let mut params = vec![$(concat!('"', stringify!($var), '"')),*];
                //let mut d_members = concat!($({format!("\t\t\t_d.{} = _args.pop();\n", stringify!($var))}),*);
                let mut d_members = String::new();
                $(write!(d_members, "\t{}: _args.shift(),\n", stringify!($var)).unwrap();)*
                //$(data_struct_d_var!(d_members, $var: $ty);)*
                write!(cls, "\nfunction {}() {{\n{}\n{}\nreturn Object.create({{\n {}}});\n}}", stringify!($name),
                    concat!(
                        "var _args = [].slice.call(arguments);"),
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
