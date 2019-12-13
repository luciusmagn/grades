#![allow(missing_docs)]
use uuid::Uuid;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

use std::process::Command;

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
	pub id: Uuid,
	pub name: String,
	pub val: GradeVal,
	pub description: Option<String>,
	pub date: DateTime<Utc>,
	pub subject: Uuid,
	pub student: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewGrade {
	pub name: String,
	pub val: GradeVal,
	pub description: Option<String>,
	pub date: DateTime<Utc>,
	pub subject: Uuid,
	pub student: Uuid,
}

impl Table for Grade {
	type Key = Uuid;
	type Value = Self;

	fn name() -> &'static str {
		"grade"
	}
}

impl NewEntry for Grade {
	type Input = NewGrade;
	type Key = <Self as Table>::Key;
	type Table = Self;

	fn create(src: NewGrade) -> (Uuid, Grade) {
		let id = Uuid::new_v4();

		(id.clone(), Grade {
			id,
			name: src.name,
			val: src.val,
			description: src.description,
			date: src.date,
			subject: src.subject,
			student: src.student,
		})
	}
}


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Kind {
	Other,
	Science,
	Humanity,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Subject {
	pub id: Uuid,
	pub name: String,
	pub description: String,
	pub year: String,
	pub grade_formula: String,
	pub kind: Kind,
	pub teacher: Uuid,
}

pub struct NewSubject {
	pub name: String,
	pub description: String,
	pub year: String,
	pub grade_formula: String,
	pub kind: Kind,
	pub teacher: Uuid,
}

impl Table for Subject {
	type Key = Uuid;
	type Value = Self;

	fn name() -> &'static str {
		"subject"
	}
}

impl NewEntry for Subject {
	type Input = NewSubject;
	type Key = <Self as Table>::Key;
	type Table = Self;

	fn create(src: NewSubject) -> (Uuid, Subject) {
		let id = Uuid::new_v4();

		(id.clone(), Subject {
			id,
			description: src.description,
			year: src.year,
			grade_formula: src.grade_formula,
			kind: src.kind,
			teacher: src.teacher,
			name: src.name,
		})
	}
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Teacher {
	pub id: Uuid,
	pub name: String,
	pub email: String,
	pub info: String,
	pub pass: String,
	pub subjects: Vec<Uuid>,
	pub pub_key: String,
	pub priv_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewTeacher {
	pub name: String,
	pub email: String,
	pub pass: String,
}

impl NewEntry for Teacher {
	type Input = NewTeacher;
	type Key = <Self as Table>::Key;
	type Table = Self;

	fn create(src: NewTeacher) -> (Uuid, Teacher) {
		let id = Uuid::new_v4();
		let keys = String::from_utf8(
			Command::new("nim/keymaster").output().expect("sad story").stdout,
		)
			.unwrap()
			.split('|')
			.map(|x| x.to_string())
			.collect::<Vec<String>>();
		let (priv_key, pub_key) = (keys[0].to_string(), keys[1].to_string());

		(id.clone(), Teacher {
			id,
			name: src.name,
			email: src.email,
			info: String::new(),
			subjects: vec![],
			pass: src.pass,
			pub_key,
			priv_key,
		})
	}
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
	pub id: Uuid,
	pub name: String,
	pub email: String,
	pub subjects: Vec<Uuid>,
	pub pass: String,
	pub pub_key: String,
	pub priv_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewStudent {
	pub name: String,
	pub email: String,
	pub pass: String,
}

impl NewEntry for Student {
	type Input = NewStudent;
	type Key = <Self as Table>::Key;
	type Table = Self;

	fn create(src: NewStudent) -> (Uuid, Student) {
		let id = Uuid::new_v4();
		let keys = String::from_utf8(
			Command::new("nim/keymaster").output().expect("sad story").stdout,
		)
			.unwrap()
			.split('|')
			.map(|x| x.to_string())
			.collect::<Vec<String>>();
		let (priv_key, pub_key) = (keys[0].to_string(), keys[1].to_string());

		(id.clone(), Student {
			id,
			name: src.name,
			email: src.email,
			subjects: vec![],
			pass: src.pass,
			pub_key,
			priv_key,
		})
	}
}

impl Table for Student {
	type Key = Uuid;
	type Value = Self;

	fn name() -> &'static str {
		"student"
	}
}
