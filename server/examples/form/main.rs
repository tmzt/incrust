#![feature(plugin)]
#![plugin(incrust_plugin)]

#[macro_use]
extern crate nickel;
#[macro_use]
extern crate incrust_macros;
#[macro_use]
extern crate examples_common;

mod models;
use models::person_js;


template! main {
    store person {
        default => ("{}");
        action SET_FIRST_NAME => ("{first_name: \"first_name\"}");
        action SET_LAST_NAME => ("{last_name: \"last_name\"}")
    }

    view root [
        p [ {"First name:  "} {(data.first_name)} ]
        p [ {"Last name:  "} {(data.last_name)} ]
        div [
            form [
                input []
                input []
            ]
        ]
    ]
}

example!(main, root, person, person_js(), "function(store) { setInterval(function() { store.dispatch({type: 'SET_FIRST_NAME'}); }, 1000); });");