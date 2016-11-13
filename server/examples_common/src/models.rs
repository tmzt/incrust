#[macro_export]
macro_rules! data_struct {
    (struct $name:ident {
        $($var: ident : $ty: ty),*
    }) => ({
                let mut cls = String::new();
                let mut members = String::new();
                $(write!(members, "\t{}: function() {{ return _d.{}; }},\n", stringify!($var), stringify!($var)).unwrap();)*

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
