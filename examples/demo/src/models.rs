use std::fmt::Write;

#[macro_use]
use examples_common::models;


pub struct Person {
    first_name: String,
    last_name: String
}

impl Person {
    pub fn first_name(&self) -> &str { &self.first_name }
    pub fn last_name(&self) -> &str { &self.last_name }
}

pub fn person_js() -> String {
    data_struct! {
        struct Person {
            first_name: String,
            last_name: String
        }
    }
}
