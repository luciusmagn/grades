/**
(c) 2015-2018 Alex Maslakov, <gildedhonour.com>, <alexmaslakoff.icu>
*
* Licensed under the Apache License, Version 2.0 (the "License");
* you may not use this file except in compliance with the License.
* You may obtain a copy of the License at
*
* http://www.apache.org/licenses/LICENSE-2.0
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific language governing permissions and
* limitations under the License.
*
* For questions and comments about this product, please see the project page at:
*
* https://github.com/GildedHonour/frank_jwt
*
*/
extern crate openssl;
extern crate serde;
extern crate base64;

#[cfg(test)]
extern crate serde_json;

#[cfg(not(test))]
extern crate serde_json;

pub mod error;

use std::str;
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Private};
use openssl::rsa::Rsa;
use openssl::sign::{Signer, Verifier};
use serde_json::Value as JsonValue;
use base64::{encode_config as b64_enc, decode_config as b64_dec};

pub use crate::error::Error;

const SEGMENTS_COUNT: usize = 3;

const STANDARD_HEADER_TYPE: &str = "JWT";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
	HS256,
	HS384,
	HS512,
	RS256,
	RS384,
	RS512,
}

impl ToString for Algorithm {
	fn to_string(&self) -> String {
		match *self {
			Algorithm::HS256 => "HS256",
			Algorithm::HS384 => "HS384",
			Algorithm::HS512 => "HS512",
			Algorithm::RS256 => "RS256",
			Algorithm::RS384 => "RS384",
			Algorithm::RS512 => "RS512",
		}
		.to_string()
	}
}

pub trait ToKey {
	fn to_key(&self) -> Result<Vec<u8>, Error>;
}

impl ToKey for String {
	fn to_key(&self) -> Result<Vec<u8>, Error> {
		Ok(self.as_bytes().to_vec())
	}
}

impl ToKey for &str {
	fn to_key(&self) -> Result<Vec<u8>, Error> {
		Ok(self.as_bytes().to_vec())
	}
}

impl ToKey for Vec<u8> {
	fn to_key(&self) -> Result<Vec<u8>, Error> {
		Ok(self.clone())
	}
}

pub fn encode<P: ToKey>(
	mut header: JsonValue,
	signing_key: &P,
	payload: &JsonValue,
	algorithm: Algorithm,
) -> Result<String, Error> {
	header["alg"] = JsonValue::String(algorithm.to_string());
	if header["typ"].is_null() {
		header["typ"] = JsonValue::String(STANDARD_HEADER_TYPE.to_owned());
	}
	let signing_input = get_signing_input(&payload, &header)?;
	let signature = match algorithm {
		Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 =>
			sign_hmac(&signing_input, signing_key, algorithm)?,
		Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 =>
			sign_rsa(&signing_input, signing_key, algorithm)?,
	};

	Ok(format!("{}.{}", signing_input, signature))
}

pub fn decode<P: ToKey>(
	encoded_token: &str,
	signing_key: &P,
	algorithm: Algorithm,
) -> Result<(JsonValue, JsonValue), Error> {
	let (header, payload, signature, signing_input) = decode_segments(encoded_token)?;
	if !verify_signature(algorithm, signing_input, &signature, signing_key)? {
		Err(Error::SignatureInvalid)
	} else {
		Ok((header, payload))
	}
}

pub fn validate_signature<P: ToKey>(
	encoded_token: &str,
	signing_key: &P,
	algorithm: Algorithm,
) -> Result<bool, Error> {
	let (signature, signing_input) = decode_signature_segments(encoded_token)?;
	verify_signature(algorithm, signing_input, &signature, signing_key)
}

fn get_signing_input(payload: &JsonValue, header: &JsonValue) -> Result<String, Error> {
	let header_json_str = serde_json::to_string(header)?;
	let encoded_header = b64_enc(header_json_str.as_bytes(), base64::URL_SAFE_NO_PAD);
	let payload_json_str = serde_json::to_string(payload)?;
	let encoded_payload = b64_enc(payload_json_str.as_bytes(), base64::URL_SAFE_NO_PAD);
	Ok(format!("{}.{}", encoded_header, encoded_payload))
}

fn sign_hmac<P: ToKey>(
	data: &str,
	key_path: &P,
	algorithm: Algorithm,
) -> Result<String, Error> {
	let stp = match algorithm {
		Algorithm::HS256 => MessageDigest::sha256(),
		Algorithm::HS384 => MessageDigest::sha384(),
		Algorithm::HS512 => MessageDigest::sha512(),
		_ => panic!("Invalid hmac algorithm"),
	};

	let key = PKey::hmac(&key_path.to_key()?)?;
	let mut signer = Signer::new(stp, &key)?;
	signer.update(data.as_bytes())?;
	let hmac = signer.sign_to_vec()?;
	Ok(b64_enc(hmac.as_slice(), base64::URL_SAFE_NO_PAD))
}

fn sign_rsa<P: ToKey>(
	data: &str,
	private_key_path: &P,
	algorithm: Algorithm,
) -> Result<String, Error> {
	let stp = match algorithm {
		Algorithm::RS256 => MessageDigest::sha256(),
		Algorithm::RS384 => MessageDigest::sha384(),
		Algorithm::RS512 => MessageDigest::sha512(),
		_ => panic!("Invalid hmac algorithm"),
	};

	let rsa = Rsa::private_key_from_pem(&private_key_path.to_key()?)?;
	let key = PKey::from_rsa(rsa)?;
	sign(data, key, stp)
}

fn sign(
	data: &str,
	private_key: PKey<Private>,
	digest: MessageDigest,
) -> Result<String, Error> {
	let mut signer = Signer::new(digest, &private_key)?;
	signer.update(data.as_bytes())?;
	let signature = signer.sign_to_vec()?;
	Ok(b64_enc(signature.as_slice(), base64::URL_SAFE_NO_PAD))
}

pub fn decode_segments(
	encoded_token: &str,
) -> Result<(JsonValue, JsonValue, Vec<u8>, String), Error> {
	let raw_segments: Vec<&str> = encoded_token.split(".").collect();
	if raw_segments.len() != SEGMENTS_COUNT {
		return Err(Error::JWTInvalid);
	}

	let header_segment = raw_segments[0];
	let payload_segment = raw_segments[1];
	let crypto_segment = raw_segments[2];
	let (header, payload) = decode_header_and_payload(header_segment, payload_segment)?;
	let signature = b64_dec(crypto_segment.as_bytes(), base64::URL_SAFE_NO_PAD)?;
	let signing_input = format!("{}.{}", header_segment, payload_segment);
	Ok((header, payload, signature.clone(), signing_input))
}

fn decode_signature_segments(encoded_token: &str) -> Result<(Vec<u8>, String), Error> {
	let raw_segments: Vec<&str> = encoded_token.split(".").collect();
	if raw_segments.len() != SEGMENTS_COUNT {
		return Err(Error::JWTInvalid);
	}

	let header_segment = raw_segments[0];
	let payload_segment = raw_segments[1];
	let crypto_segment = raw_segments[2];
	let signature = b64_dec(crypto_segment.as_bytes(), base64::URL_SAFE_NO_PAD)?;
	let signing_input = format!("{}.{}", header_segment, payload_segment);
	Ok((signature.clone(), signing_input))
}

fn decode_header_and_payload(
	header_segment: &str,
	payload_segment: &str,
) -> Result<(JsonValue, JsonValue), Error> {
	let b64_to_json = |seg| -> Result<JsonValue, Error> {
		serde_json::from_slice(b64_dec(seg, base64::URL_SAFE_NO_PAD)?.as_slice())
			.map_err(Error::from)
	};

	let header_json = b64_to_json(header_segment)?;
	let payload_json = b64_to_json(payload_segment)?;
	Ok((header_json, payload_json))
}

fn sign_hmac2(data: &str, key: &Vec<u8>, algorithm: Algorithm) -> Result<Vec<u8>, Error> {
	let stp = match algorithm {
		Algorithm::HS256 => MessageDigest::sha256(),
		Algorithm::HS384 => MessageDigest::sha384(),
		Algorithm::HS512 => MessageDigest::sha512(),
		_ => panic!("Invalid HMAC algorithm"),
	};

	let pkey = PKey::hmac(key)?;
	let mut signer = Signer::new(stp, &pkey)?;
	signer.update(data.as_bytes())?;
	signer.sign_to_vec().map_err(Error::from)
}

fn verify_signature<P: ToKey>(
	algorithm: Algorithm,
	signing_input: String,
	signature: &[u8],
	public_key: &P,
) -> Result<bool, Error> {
	match algorithm {
		Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
			let signature2 =
				sign_hmac2(&signing_input, &public_key.to_key()?, algorithm)?;
			Ok(secure_compare(signature, &signature2))
		}
		Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 => {
			let rsa = Rsa::public_key_from_pem(&public_key.to_key()?)?;
			let key = PKey::from_rsa(rsa)?;

			let digest = get_sha_algorithm(algorithm);
			let mut verifier = Verifier::new(digest, &key)?;
			verifier.update(signing_input.as_bytes())?;
			verifier.verify(&signature).map_err(Error::from)
		}
	}
}

fn get_sha_algorithm(alg: Algorithm) -> MessageDigest {
	match alg {
		Algorithm::RS256 => MessageDigest::sha256(),
		Algorithm::RS384 => MessageDigest::sha384(),
		Algorithm::RS512 => MessageDigest::sha512(),
		_ => panic!("Invalid RSA algorithm"),
	}
}

fn secure_compare(a: &[u8], b: &[u8]) -> bool {
	if a.len() != b.len() {
		return false;
	}

	let mut res = 0_u8;
	for (&x, &y) in a.iter().zip(b.iter()) {
		res |= x ^ y;
	}

	res == 0
}
