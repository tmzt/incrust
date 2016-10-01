
#[macro_export]
macro_rules! define_store_action {
    ($w: expr, $store: ident, action $name:ident => ($e:expr)) => (
        write!($w, "\t\tcase '{}': return {};\n",
            stringify!($name),
            format!("{{ {}: {} }}",
                stringify!($store),
                format!("(state.{})", stringify!($e))
            )
        )
    )
}

#[macro_export]
macro_rules! define_store {
    ($w: expr, store $store:ident {
        default => ($def:expr);
        $(action $act: ident => ($e:expr));*
    }) => ({
        let mut actions = String::new();
        $(define_store_action!(actions, $store, action $act => ($e));)*
        write!(actions, "\t\tdefault: return state;\n");

        let default_value = format!("\tstate = state || {{ {}: {} }};\n", stringify!($store), stringify!($def));
        let switch = format!("\tswitch (action.type) {{\n {}\n\t}};", actions);

        let body = format!("\n{}\n{}", default_value, switch);
        let create_store = format!("Redux.createStore(function store_handler_{}(state, action) {{ {} }})",
            stringify!($store), body);
        let store_fn = format!("\nfunction create_store_{}() {{return {}}}", stringify!($store), create_store);
        write!($w, "{}", store_fn);
    })
}

#[macro_export]
macro_rules! define_stores {
    ($(store $store:ident {
        default => ($def:expr);
        $(action $act: ident => ($e:expr));*
    });*) => ({
        let mut stores = String::new();
        $(define_store!(stores, store $store { default => ($def); $(action $act => ($e));* });)*

        stores
    })
}
