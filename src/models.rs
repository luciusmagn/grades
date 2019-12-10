#![allow(missing_docs)]
use uuid::Uuid;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

use crate::db::{
	Database,
	Table,
	NewEntry,
	NewEntryPartial,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GradeVal {
	Regular(f32),
	Bonus(u32),
	Penalisation(u32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Grade {
	pub name: String,
	pub val: GradeVal,
	pub description: Option<String>,
	pub date: DateTime<Utc>,
	pub subject: Uuid,
}

impl Table for Grade {
	type Key = Uuid;
	type Value = Self;

	fn name() -> &'static str {
		"grade"
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Subject {
	pub name: String,
	pub description: String,
	pub year: String,
	pub grade_formula: String,
}


impl Table for Subject {
	type Key = Uuid;
	type Value = Self;

	fn name() -> &'static str {
		"subject"
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Teacher {
	pub name: String,
	pub email: String,
	pub subjects: Vec<Uuid>,
}

impl Table for Teacher {
	type Key = Uuid;
	type Value = Self;

	fn name() -> &'static str {
		"teacher"
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Student {
	pub name: String,
	pub email: String,
	pub subjects: Vec<Uuid>,
}

impl Table for Student {
	type Key = Uuid;
	type Value = Self;

	fn name() -> &'static str {
		"student"
	}
}
