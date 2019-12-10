//! hlavní modul
#![allow(proc_macro_derive_resolution_fallback)]
#![allow(non_upper_case_globals)]
#![deny(missing_docs)]
#![feature(associated_type_defaults, try_trait)]

#[macro_use]
extern crate lazy_static;

extern crate serde_cbor;
extern crate rocket;
extern crate chrono;
extern crate serde;
extern crate uuid;
extern crate sled;

mod db;
mod models;

fn main() {
    println!("Hello, world!");
}
