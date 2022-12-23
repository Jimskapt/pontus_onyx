use std::collections::BTreeMap;

use rand::{seq::IteratorRandom, Rng};

use crate::BearerAccess;

pub struct Database<E: crate::Engine> {
	engine: E,
	users: Vec<crate::User>,
	listeners: Vec<Box<dyn crate::Listener>>,
	settings: crate::DatabaseSettings,
}

impl<E: crate::Engine> Database<E> {
	pub fn new(engine: E) -> Self {
		Self {
			engine,
			users: vec![],
			listeners: vec![],
			settings: crate::DatabaseSettings::default(),
		}
	}
	pub fn create_user(&mut self, username: impl Into<String>, password: &mut str) {
		let username = username.into();

		match self.users.iter_mut().find(|user| user.username == username) {
			Some(user) => {
				user.password = String::from(password);
				user.tokens = BTreeMap::new();
			}
			None => {
				self.users.push(crate::User {
					username,
					password: String::from(password),
					tokens: BTreeMap::new(),
				});
			}
		}
	}
	pub fn generate_token(
		&mut self,
		username: impl Into<String>,
		password: &mut str,
		bearers: impl Into<String>,
	) -> Result<crate::Token, AuthenticateError> {
		let username = username.into();
		let bearers = bearers.into();

		if let Some(user) = self.users.iter_mut().find(|user| user.username == username) {
			if user.password == *password {
				let mut converted_bearers = vec![];

				for bearer in bearers.split(',') {
					match BearerAccess::try_from(bearer) {
						Ok(bearer) => {
							converted_bearers.push(bearer);
						}
						Err(error) => {
							return Err(AuthenticateError::WrongBearerSyntax(
								String::from(bearer),
								error,
							));
						}
					}
				}

				let mut name = String::new();
				let mut rng_limit = rand::thread_rng();
				for _ in 1..rng_limit.gen_range(128..256) {
					let mut rng_item = rand::thread_rng();
					name.push(
						crate::ACCESS_TOKEN_ALPHABET
							.chars()
							.choose(&mut rng_item)
							.unwrap(),
					);
				}
				let name = crate::Token::from(name);

				user.tokens
					.insert(name.clone(), crate::TokenMetadata::new(converted_bearers));

				Ok(name)
			} else {
				return Err(AuthenticateError::WrongPassword);
			}
		} else {
			return Err(AuthenticateError::UserNotFound(username.clone()));
		}
	}
	pub fn register_listener(&mut self, listener: Box<dyn crate::Listener>) {
		self.listeners.push(listener);
	}
}

#[cfg(test)]
struct EmptyEngineForTests {}

#[cfg(test)]
static EMPTY_ENGINE_PASS_RESPONSE: &str = "TEST ENGINE : NOT EXPECTED FOR PRODUCTION USE";

#[cfg(test)]
impl crate::Engine for EmptyEngineForTests {
	fn perform(&mut self, _: &crate::Request) -> crate::EngineResponse {
		crate::EngineResponse::InternalError(String::from(EMPTY_ENGINE_PASS_RESPONSE))
	}

	fn new_for_tests() -> Self {
		Self {}
	}
	fn root_for_tests(&self) -> crate::Item {
		crate::Item::Folder {
			etag: None,
			last_modified: None,
		}
	}
}

#[test]
fn generate_token() {
	let mut database = Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests());
	database.create_user("my_user", &mut String::from("my_password"));

	assert!(database
		.generate_token("", &mut String::from("my_password"), "my_access:rw")
		.is_err());
	assert!(database
		.generate_token(
			"WRONG_user",
			&mut String::from("my_password"),
			"my_access:rw"
		)
		.is_err());

	assert!(database
		.generate_token(
			"my_user",
			&mut String::from("WRONG_password"),
			"my_access:rw"
		)
		.is_err());
	assert!(database
		.generate_token("my_user", &mut String::from(""), "my_access:rw")
		.is_err());

	assert!(database
		.generate_token(
			"my_user",
			&mut String::from("my_password"),
			"WRONG/ACCESS:rw"
		)
		.is_err());
	assert!(database
		.generate_token("my_user", &mut String::from("my_password"), "")
		.is_err());

	assert!(database
		.generate_token(
			"WRONG_user",
			&mut String::from("WRONG_password"),
			"WRONG/ACCESS:rw"
		)
		.is_err());

	assert!(database
		.generate_token("my_user", &mut String::from("my_password"), "my_access:rw")
		.is_ok());
	assert!(!database
		.generate_token("my_user", &mut String::from("my_password"), "my_access:rw")
		.unwrap()
		.0
		.is_empty());
}

#[test]
fn should_not_list_public() {
	assert_eq!(
		Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests())
			.perform(crate::Request::get("public/").unwrap())
			.status,
		crate::ResponseStatus::Unallowed(AccessError::CanNotListPublic)
	);
}

#[test]
fn should_not_list_public_subfolder() {
	assert_eq!(
		Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests())
			.perform(crate::Request::get("public/folder/").unwrap())
			.status,
		crate::ResponseStatus::Unallowed(AccessError::CanNotListPublic)
	);
}

#[test]
fn should_pass_public_get() {
	assert_eq!(
		Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests())
			.perform(crate::Request::get("public/document.txt").unwrap())
			.status,
		crate::ResponseStatus::Performed(crate::EngineResponse::InternalError(String::from(
			EMPTY_ENGINE_PASS_RESPONSE
		)))
	);
}

#[test]
fn should_pass_public_get_subfolder() {
	assert_eq!(
		Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests())
			.perform(crate::Request::get("public/folder/document.txt").unwrap())
			.status,
		crate::ResponseStatus::Performed(crate::EngineResponse::InternalError(String::from(
			EMPTY_ENGINE_PASS_RESPONSE
		)))
	);
}

#[test]
fn should_not_pass_without_token() {
	assert_eq!(
		Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests())
			.perform(crate::Request::get("folder_a/").unwrap())
			.status,
		crate::ResponseStatus::Unallowed(AccessError::MissingToken)
	);
}

#[test]
fn should_pass_with_right_token() {
	let mut database = Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:r")
		.unwrap();

	assert_eq!(
		database
			.perform(crate::Request::get("folder_a/").unwrap().token(token))
			.status,
		crate::ResponseStatus::Performed(crate::EngineResponse::InternalError(String::from(
			EMPTY_ENGINE_PASS_RESPONSE
		)))
	);
}

#[test]
fn should_not_pass_with_wrong_token() {
	let mut database = Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_b:r")
		.unwrap();

	assert_eq!(
		database
			.perform(crate::Request::get("folder_a/").unwrap().token(token))
			.status,
		crate::ResponseStatus::Unallowed(crate::AccessError::NotValidToken(vec![
			crate::TokenValidityError::RequestError(crate::RequestValidityError::OutOfModuleScope)
		]))
	);
}

#[test]
fn should_not_pass_with_token_but_wrong_method() {
	let mut database = Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:r")
		.unwrap();

	assert_eq!(
		database
			.perform(
				crate::Request::put("folder_a/document.json")
					.unwrap()
					.token(token)
					.item(crate::Item::Document {
						etag: None,
						last_modified: None,
						content: Some(br#"{"key": "value"}"#.into()),
						content_type: Some("application/json".into())
					})
			)
			.status,
		crate::ResponseStatus::Unallowed(crate::AccessError::NotValidToken(vec![
			crate::TokenValidityError::RequestError(crate::RequestValidityError::UnallowedMethod)
		]))
	);
}

impl<E: crate::Engine> Database<E> {
	pub fn perform(&mut self, request: impl Into<crate::Request>) -> crate::Response {
		let request = request.into();

		let status = match self.is_allowed(&request) {
			Ok(()) => {
				let engine_response = self.engine.perform(&request);

				if engine_response.has_muted_database() {
					for listener in &mut self.listeners {
						listener.receive(crate::Event::build_from(&request, &engine_response));
					}
				}

				crate::ResponseStatus::Performed(engine_response)
			}
			Err(error) => crate::ResponseStatus::Unallowed(error),
		};

		crate::Response { request, status }
	}
	fn is_allowed(&self, request: &crate::Request) -> Result<(), AccessError> {
		if request.path.starts_with("/public/") {
			if request.path.target_is_document() {
				return Ok(());
			} else {
				return Err(AccessError::CanNotListPublic);
			}
		} else {
			match &request.token {
				Some(_) => return self.check_token(request),
				None => {
					return Err(AccessError::MissingToken);
				}
			}
		}
	}
	fn check_token(&self, request: &crate::Request) -> Result<(), AccessError> {
		let token = crate::Token::from(request.token.as_ref().unwrap().0.clone()); // already checked in calling function so unwrap is OK

		match self
			.users
			.iter()
			.find(|user| user.tokens.contains_key(&token))
		{
			Some(user) => {
				let token_metadata = user.tokens.get(&token).unwrap();
				token_metadata
					.check(self.settings.token_lifetime_seconds, request)
					.map_err(AccessError::NotValidToken)
			}
			None => Err(AccessError::UnknownToken),
		}
	}
}

impl<E: crate::Engine> crate::Listener for Database<E> {
	fn receive(&mut self, event: crate::Event) -> crate::Response {
		todo!()
	}
}

#[derive(Debug, PartialEq)]
pub enum AccessError {
	CanNotListPublic,
	MissingToken,
	UnknownToken,
	NotValidToken(Vec<crate::TokenValidityError>),
}

#[derive(Debug, PartialEq)]
pub enum AuthenticateError {
	WrongBearerSyntax(String, crate::BearerAccessConvertError),
	UserNotFound(String),
	WrongPassword,
}
