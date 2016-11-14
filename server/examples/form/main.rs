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
        default => Person { first_name: (""), last_name: ("") };
        action SET_FIRST_NAME => Person { first_name: ("first_name")};
        action SET_LAST_NAME => Person { last_name: ("last_name")};
        action SET_BOTH_NAMES => Person { first_name: ("Front-end"), last_name: ("User") }
    }

    view root [
        p [ {"First name:  "} {(store.first_name)} ]
        p [ {"Last name:  "} {(store.last_name)} ]
        div [
            form [
                input []
                input []
            ]
        ]
    ]
}

example!(main, root, person, person_js(), "function start(store) { setInterval(function() { store.dispatch({type: 'SET_BOTH_NAMES'}); }, 1000); };");