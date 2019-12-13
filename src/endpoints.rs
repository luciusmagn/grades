use rocket_contrib::json::Json;

use crate::db::{Database, NewEntry, NewEntryPartial};
use crate::models::{
	Student,
	NewStudent,
	Teacher,
	NewTeacher,
	Grade,
	NewGrade,
	Subject,
	NewSubject,
};

#[get("/subjects")]
pub(crate) fn subjects(db: Database<Subject>) -> Json<Vec<Subject>> {
	Json(db
		.read()
		.iter()
		.map(|(_, x)| x)
		.collect::<Vec<_>>())
}

#[get("/teachers")]
pub(crate) fn teachers(db: Database<Teacher>) -> Json<Vec<Teacher>> {
	Json(db
		.read()
		.iter()
		.map(|(_, x)| x)
		.collect::<Vec<_>>())
}

#[get("/grades")]
pub(crate) fn grades(db: Database<Grade>) -> Json<Vec<Grade>> {
	Json(db
		.read()
		.iter()
		.map(|(_, x)| x)
		.collect::<Vec<_>>())
}

#[post("/register_student", format = "application/json", data = "<input>")]
pub(crate) fn register_student(input: Json<NewStudent>) -> Option<()> {
	Student::create(input.clone())
		.save()
		.map(|_| ())
		.ok()
}

#[post("/register_teacher", format = "application/json", data = "<input>")]
pub(crate) fn register_teacher(input: Json<NewTeacher>) -> Option<()> {
	Teacher::create(input.clone())
		.save()
		.map(|_| ())
		.ok()
}

//#[post("/login_student")]
