use uuid::Uuid;
use rocket_contrib::json::Json;
use serde_json::{json, Value, value};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, NaiveDateTime};

use std::str::FromStr;

use crate::auth::AuthToken;
use crate::db::{Database, NewEntry, NewEntryPartial};
use crate::models::{
	Student,
	NewStudent,
	Teacher,
	NewTeacher,
	Grade,
	NewGrade,
	GradeVal,
	Subject,
	NewSubject,
};

use rejwt::{
	encode,
	Algorithm,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginForm {
	pub email: String,
	pub pass: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NewGradeForm {
	pub typ: String,
	pub name: String,
	pub date: String,
	pub subject: Uuid,
	pub number: String,
	pub student: Uuid,
	pub description: String,
}

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
		.inspect(|(_, x)| println!("{:?}", x))
		.map(|(_, x)| x)
		.collect::<Vec<_>>())
}

#[get("/students")]
pub(crate) fn students(db: Database<Student>) -> Json<Vec<Student>> {
	Json(db
		.read()
		.iter()
		.inspect(|(_, x)| println!("{:?}", x))
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

#[get("/me")]
pub(crate) fn me(teachers: Database<Teacher>, students: Database<Student>, info: AuthToken) -> Option<Json<Value>> {
	match info.typ.as_ref() {
		"teacher" => Some(Json(value::to_value(teachers.read().get(&info.id)?).unwrap())),
		"student" => Some(Json(value::to_value(students.read().get(&info.id)?).unwrap())),
		_ => None
	}
}

#[post("/my_description", format = "application/json", data = "<input>")]
pub(crate) fn my_description(mut db: Database<Teacher>, input: Json<String>, info: AuthToken) -> Option<()> {
	println!("{}", *input);
	let _ = db
		.write()
		.update::<_, Teacher, _>(info.id, |c| {
			println!("{:?}", c);
			c.map(|mut x| {
				x.info = input.clone();
				x
			})
		});

	Some(())
}

#[post("/register_student", format = "application/json", data = "<input>")]
pub(crate) fn register_student(input: Json<NewStudent>, mut _db: Database<Student>) -> Option<()> {
	Student::create(input.clone())
		.save()
		.map(|_| ())
		.ok()
}

#[post("/register_teacher", format = "application/json", data = "<input>")]
pub(crate) fn register_teacher(input: Json<NewTeacher>, mut _db: Database<Teacher>) -> Option<()> {
	Teacher::create(input.clone())
		.save()
		.map(|_| ())
		.ok()
}

#[post("/login_student", format = "application/json", data = "<input>")]
pub(crate) fn login_student(input: Json<LoginForm>, db: Database<Student>) -> Option<Json<String>> {
	let u = db
		.read()
		.iter()
		.find_map(|(_, x)| if x.email == input.email && x.pass == input.pass {
			Some(x)
		} else { None })?;

	let header = json!({ "kid": u.id.to_hyphenated().to_string() });
	let body = AuthToken::new(u.id, "student".to_string()).make();

	Some(Json(encode(header, &u.priv_key, &body, Algorithm::RS256).unwrap()))
}

#[post("/login_teacher", format = "application/json", data = "<input>")]
pub(crate) fn login_teacher(input: Json<LoginForm>, db: Database<Teacher>) -> Option<Json<String>> {
	println!("{:?}", input);
	println!("{}", db.read().iter().count());

	let u = db
		.read()
		.iter()
		.inspect(|e| println!("{:?}", e))
		.find_map(|(_, x)| if x.email == input.email && x.pass == input.pass {
			Some(x)
		} else { None })?;

	let header = json!({ "kid": u.id.to_hyphenated().to_string() });
	let body = AuthToken::new(u.id, "teacher".to_string()).make();

	Some(Json(encode(header, &u.priv_key, &body, Algorithm::RS256).unwrap()))
}

#[post("/subject", format = "application/json", data = "<input>")]
pub(crate) fn new_subject(input: Json<NewSubject>, mut _db: Database<Subject>, info: AuthToken) -> Option<()> {
	Subject::create(input.clone())
		.and_modify(|mut x| x.teacher = info.id)
		.save()
		.map(|_| ())
		.ok()
}

#[post("/grade", format = "application/json", data = "<input>")]
pub(crate) fn new_grade(input: Json<NewGradeForm>, mut _db: Database<Grade>, _info: AuthToken) -> Option<()> {
	println!("{:?}", input);
	let new_grade = NewGrade {
		name: input.name.clone(),
		val: match input.typ.as_str() {
			"Regular" => GradeVal::Regular(f32::from_str(&input.number).unwrap()),
			"Bonus" => GradeVal::Bonus(i32::from_str(&input.number).unwrap()),
			"Penalisation" => GradeVal::Penalisation(i32::from_str(&input.number).unwrap()),
			_ => GradeVal::Regular(f32::from_str(&input.number).unwrap()),
		},
		description: Some(input.description.clone()),
		date: DateTime::<Utc>::from_utc(
			NaiveDateTime::parse_from_str(
					&format!("{} 0:0:0", input.date), 
					"%Y-%m-%d %H:%M:%S")
				.unwrap(),
			Utc),
		student: input.student.clone(),
		subject: input.subject.clone(),
	};
	Grade::create(new_grade)
		.save()
		.map(|_| ())
		.ok()
}

#[post("/subject/sign_up", format = "application/json", data = "<input>")]
pub(crate) fn sign_up(input: Json<Uuid>,  mut db: Database<Student>, info: AuthToken) -> Option<()> {
	println!("{:?} {:?}", input, info);
	let _ = db
		.write()
		.update::<_, Student, _>(info.id, |c| {
			println!("{:?}", c);
			c.map(|mut x| {
				x.subjects.push(*input);
				x
			})
		});

	Some(())
}
