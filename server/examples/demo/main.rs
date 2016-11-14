#![feature(plugin)]
#![plugin(incrust_plugin)]

#[macro_use]
extern crate nickel;
#[macro_use]
extern crate incrust_macros;
#[macro_use]
extern crate examples_common;


template! main {
    store counter {
        default => (0);
        action INCREMENT => (counter + 1);
        action DECREMENT => (counter - 1)
    }

    view root [
        div [
            h1 [ {"Counter: "}{store} ]
        ]
    ]
}

example!(main, root, counter, r"
    function start(store) {
        setInterval(function() { store.dispatch({type: 'INCREMENT'}); }, 1000);
    };");
