//! hlavní modul
#![allow(proc_macro_derive_resolution_fallback)]
#![allow(non_upper_case_globals)]
#![deny(missing_docs)]
#![feature(associated_type_defaults,ecl_macro, proc_macro_hygiene)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket;

extern crate serde_cbor;
extern crate chrono;
extern crate serde;
extern crate uuid;
extern crate sled;

mod db;
mod models;
mod endpoints;

use std::path::{PathBuf, Path};
use rocket::response::NamedFile;

#[get("/")]
fn index() -> NamedFile {
	NamedFile::open("www/index.html").expect("index.html not found")
}

/// returns static files of frontend
#[get("/static/<name..>")]
fn frontend(name: PathBuf) -> Option<NamedFile> {
	NamedFile::open(Path::new("static/").join(name)).ok()
}

fn main() {
	rocket::ignite()
		.mount("/", routes![
			index,
			frontend,
		])
		.launch();
}
