//! hlavní modul
#![allow(proc_macro_derive_resolution_fallback)]
#![allow(non_upper_case_globals)]
#![deny(missing_docs)]
#![feature(associated_type_defaults, decl_macro, proc_macro_hygiene)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket;

extern crate serde_cbor;
extern crate chrono;
extern crate dotenv;
extern crate serde;
extern crate uuid;
extern crate sled;

extern crate rejwt;

mod db;
mod auth;
mod models;
mod endpoints;

use std::path::{PathBuf, Path};
use rocket::response::NamedFile;

#[get("/")]
fn index() -> NamedFile {
	NamedFile::open("index.html").expect("index.html not found")
}

/// returns static files of frontend
#[get("/pkg/<name..>")]
fn frontend(name: PathBuf) -> Option<NamedFile> {
	NamedFile::open(Path::new("pkg/").join(name)).ok()
}

fn main() {
	dotenv::dotenv().ok();

	rocket::ignite()
		.mount("/", routes![
			index,
			frontend,
			endpoints::grades,
			endpoints::subjects,
			endpoints::teachers,
			endpoints::register_student,
			endpoints::register_teacher,
		])
		.launch();
}
