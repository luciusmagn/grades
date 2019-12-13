//! Modul obsahující věci týkající se autentifikace
use crate::db::Database;
use crate::models::{Teacher, Student};

use uuid::Uuid;
use serde::{Deserialize, Serialize};

use rocket::Outcome;
use rocket::Request;
use rocket::http::Status;
use rocket::request::FromRequest;

use chrono::Utc;

/// JWT pro autorizaci atd.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuthToken {
	/// Issuer JWT - remedias
	pub iss: String,
	/// Doba vypršení
	pub exp: i64,
	/// Identifikátor uživatele
	pub id: Uuid,
	/// typ uživatele
	pub typ: String,
}

impl AuthToken {
	/// Vytvoří nový authtoken z usera
	pub fn new(id: Uuid, typ: String) -> AuthToken {
		let now = Utc::now().timestamp() + (69 * 60);

		AuthToken {
			iss:       "Znamky".to_string(),
			exp:       now,
			id:        id,
			typ:       typ,
		}
	}


	/// Nastaví timestamp na vypršení tokenu
	pub fn exp(mut self, exp: i64) -> AuthToken {
		self.exp = exp;
		self
	}

	/// Zkonvertuje token na json
	pub fn make(&self) -> serde_json::Value {
		serde_json::to_value(&self).unwrap()
	}
}

impl<'a, 'r> FromRequest<'a, 'r> for AuthToken {
	type Error = String;

	fn from_request(
		request: &'a Request<'r>,
	) -> rocket::request::Outcome<Self, Self::Error> {
		let keys: Vec<_> = request.headers().get("Authorization").collect();
		println!("lel");
		match keys.get(0).unwrap_or(&"").split(' ').nth(1) {
			Some(ref token) => {
				let header = rejwt::decode_segments(&token).unwrap().0;

				if let Some(key_id) = header.get("kid") {
					let user_id = Uuid::parse_str(key_id.as_str().unwrap()).ok().unwrap();
					let teachers = Database::<Teacher>::open().unwrap();
					let students = Database::<Student>::open().unwrap();
					
					let key = match teachers.read().get(&user_id) {
						Some(u) => u.pub_key.clone(),
						None => match students.read().get(&user_id) {
							Some(u) => u.pub_key.clone(),
							None => unreachable!(),
						}
					};

					match rejwt::decode(&token, &key, rejwt::Algorithm::RS256) {
						Ok(t) => match serde_json::from_value::<AuthToken>(t.1) {
							Ok(tok) => {
								let now = Utc::now().timestamp();

								println!("lel 2");
								if now > tok.exp {
									Outcome::Failure((
										Status::ImATeapot,
										"token has expired".to_string(),
									))
								} else if global_auth(&tok).is_some() {
									Outcome::Success(tok)
								} else {
									Outcome::Failure((
										Status::Forbidden,
										"invalid token"
											.to_string(),
									))
								}
							}
							Err(_) => Outcome::Failure((
								Status::UnprocessableEntity,
								"token contains invalid data".to_string(),
							)),
						},
						Err(_) => Outcome::Failure((
							Status::Unauthorized,
							"couldn't verify token".to_string(),
						)),
					}
				} else {
					Outcome::Failure((
						Status::BadRequest,
						"invalid JWT header - missing kid".to_string(),
					))
				}
			}
			x => {
				println!("{:?}", x);
				Outcome::Failure((
					Status::BadRequest,
					"invalid authorization header".to_string(),
				))
			}
		}
	}
}


/// Autentifikace
pub fn global_auth(info: &AuthToken) -> Option<(Uuid, String)> {
	let teachers  = Database::<Teacher>::open()?;
	let students  = Database::<Student>::open()?;

	if let Some(t) = teachers.read().get(&info.id) {
		Some((t.id, "teacher".into()))
	} else {
		Some((students.read().get(&info.id)?.id, "student".into()))
	}
}

