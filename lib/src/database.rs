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
#[async_trait::async_trait]
impl crate::Engine for EmptyEngineForTests {
	async fn perform(&mut self, request: &crate::Request) -> crate::EngineResponse {
		if request.path == "folder_a/existing.json".try_into().unwrap() {
			return crate::EngineResponse::GetSuccessDocument(crate::Item::Document {
				etag: Some("DOCUMENT_ETAG".into()),
				last_modified: Some(
					time::OffsetDateTime::from_unix_timestamp(1000)
						.unwrap()
						.into(),
				),
				content: Some(b"DOCUMENT_CONTENT".into()),
				content_type: Some("DOCUMENT_CONTENT_TYPE".into()),
			});
		} else if request.path == "folder_a/not_existing.json".try_into().unwrap() {
			return crate::EngineResponse::NotFound;
		}

		return crate::EngineResponse::InternalError(String::from(EMPTY_ENGINE_PASS_RESPONSE));
	}

	fn new_for_tests() -> Self {
		Self {}
	}
	fn root_for_tests(&self) -> BTreeMap<crate::ItemPath, crate::Item> {
		BTreeMap::new()
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

#[tokio::test]
async fn should_not_list_public() {
	assert_eq!(
		Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests())
			.perform(crate::Request::get(
				crate::ItemPath::try_from("public/").unwrap()
			))
			.await
			.status,
		crate::ResponseStatus::Unallowed(AccessError::CanNotListPublic)
	);
}

#[tokio::test]
async fn should_not_list_public_subfolder() {
	assert_eq!(
		Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests())
			.perform(crate::Request::get(
				crate::ItemPath::try_from("public/folder/").unwrap()
			))
			.await
			.status,
		crate::ResponseStatus::Unallowed(AccessError::CanNotListPublic)
	);
}

#[tokio::test]
async fn should_pass_public_get() {
	assert_eq!(
		Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests())
			.perform(crate::Request::get(
				crate::ItemPath::try_from("public/document.txt").unwrap()
			))
			.await
			.status,
		crate::ResponseStatus::Performed(crate::EngineResponse::InternalError(String::from(
			EMPTY_ENGINE_PASS_RESPONSE
		)))
	);
}

#[tokio::test]
async fn should_pass_public_get_subfolder() {
	assert_eq!(
		Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests())
			.perform(crate::Request::get(
				crate::ItemPath::try_from("public/folder/document.txt").unwrap()
			))
			.await
			.status,
		crate::ResponseStatus::Performed(crate::EngineResponse::InternalError(String::from(
			EMPTY_ENGINE_PASS_RESPONSE
		)))
	);
}

#[tokio::test]
async fn should_not_pass_without_token() {
	assert_eq!(
		Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests())
			.perform(crate::Request::get(
				crate::ItemPath::try_from("folder_a/").unwrap()
			))
			.await
			.status,
		crate::ResponseStatus::Unallowed(AccessError::MissingToken)
	);
}

#[tokio::test]
async fn should_pass_with_right_token() {
	let mut database = Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:r")
		.unwrap();

	assert_eq!(
		database
			.perform(
				crate::Request::get(crate::ItemPath::try_from("folder_a/").unwrap()).token(token)
			)
			.await
			.status,
		crate::ResponseStatus::Performed(crate::EngineResponse::InternalError(String::from(
			EMPTY_ENGINE_PASS_RESPONSE
		)))
	);
}

#[tokio::test]
async fn should_not_pass_with_wrong_token() {
	let mut database = Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_b:r")
		.unwrap();

	assert_eq!(
		database
			.perform(
				crate::Request::get(crate::ItemPath::try_from("folder_a/").unwrap()).token(token)
			)
			.await
			.status,
		crate::ResponseStatus::Unallowed(crate::AccessError::NotValidToken(vec![
			crate::TokenValidityError::RequestError(crate::RequestValidityError::OutOfModuleScope)
		]))
	);
}

#[tokio::test]
async fn should_not_pass_with_token_but_wrong_method() {
	let mut database = Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:r")
		.unwrap();

	assert_eq!(
		database
			.perform(
				crate::Request::put(crate::ItemPath::try_from("folder_a/document.json").unwrap())
					.token(token)
					.item(crate::Item::Document {
						etag: None,
						last_modified: None,
						content: Some(br#"{"key": "value"}"#.into()),
						content_type: Some("application/json".into())
					})
			)
			.await
			.status,
		crate::ResponseStatus::Unallowed(crate::AccessError::NotValidToken(vec![
			crate::TokenValidityError::RequestError(crate::RequestValidityError::UnallowedMethod)
		]))
	);
}

#[tokio::test]
async fn get_no_if_match() {
	let mut database = Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:r")
		.unwrap();

	assert_eq!(
		database
			.perform(
				crate::Request::get(crate::ItemPath::try_from("folder_a/existing.json").unwrap())
					.token(token)
					.add_limit(crate::Limit::IfMatch("WRONG_ETAG".into()))
			)
			.await
			.status,
		crate::ResponseStatus::NoIfMatch("DOCUMENT_ETAG".into())
	);
}

#[tokio::test]
async fn get_if_match() {
	let mut database = Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:r")
		.unwrap();

	assert_eq!(
		database
			.perform(
				crate::Request::get(crate::ItemPath::try_from("folder_a/existing.json").unwrap())
					.token(token)
					.add_limit(crate::Limit::IfMatch("DOCUMENT_ETAG".into()))
			)
			.await
			.status,
		crate::ResponseStatus::Performed(crate::EngineResponse::GetSuccessDocument(
			crate::Item::Document {
				etag: Some("DOCUMENT_ETAG".into()),
				last_modified: Some(
					time::OffsetDateTime::from_unix_timestamp(1000)
						.unwrap()
						.into()
				),
				content: Some(b"DOCUMENT_CONTENT".into()),
				content_type: Some("DOCUMENT_CONTENT_TYPE".into())
			}
		))
	);
}

#[tokio::test]
async fn get_if_none_match() {
	let mut database = Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:r")
		.unwrap();

	assert_eq!(
		database
			.perform(
				crate::Request::get(crate::ItemPath::try_from("folder_a/existing.json").unwrap())
					.token(token)
					.add_limit(crate::Limit::IfNoneMatch("DOCUMENT_ETAG".into()))
			)
			.await
			.status,
		crate::ResponseStatus::IfNoneMatch("DOCUMENT_ETAG".into())
	);
}

#[tokio::test]
async fn get_no_if_none_match() {
	let mut database = Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:r")
		.unwrap();

	assert_eq!(
		database
			.perform(
				crate::Request::get(crate::ItemPath::try_from("folder_a/existing.json").unwrap())
					.token(token)
					.add_limit(crate::Limit::IfNoneMatch("ANOTHER_DOCUMENT_ETAG".into()))
			)
			.await
			.status,
		crate::ResponseStatus::Performed(crate::EngineResponse::GetSuccessDocument(
			crate::Item::Document {
				etag: Some("DOCUMENT_ETAG".into()),
				last_modified: Some(
					time::OffsetDateTime::from_unix_timestamp(1000)
						.unwrap()
						.into()
				),
				content: Some(b"DOCUMENT_CONTENT".into()),
				content_type: Some("DOCUMENT_CONTENT_TYPE".into())
			}
		))
	);
}

#[tokio::test]
async fn get_if_none_match_all() {
	let mut database = Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:r")
		.unwrap();

	assert_eq!(
		database
			.perform(
				crate::Request::get(crate::ItemPath::try_from("folder_a/existing.json").unwrap())
					.token(token)
					.add_limit(crate::Limit::IfNoneMatch("*".into()))
			)
			.await
			.status,
		crate::ResponseStatus::IfNoneMatch("DOCUMENT_ETAG".into())
	);
}

#[tokio::test]
async fn put_content_not_changed() {
	let mut database = Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:rw")
		.unwrap();

	assert_eq!(
		database
			.perform(
				crate::Request::put(crate::ItemPath::try_from("folder_a/existing.json").unwrap())
					.token(token)
					.item(crate::Item::Document {
						etag: None,
						last_modified: None,
						content: Some("DOCUMENT_CONTENT".into()),
						content_type: Some("DOCUMENT_CONTENT_TYPE".into())
					})
			)
			.await
			.status,
		crate::ResponseStatus::ContentNotChanged
	);
}

#[tokio::test]
async fn put_folder_path() {
	let mut database = Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:rw")
		.unwrap();

	assert_eq!(
		database
			.perform(
				crate::Request::put(crate::ItemPath::try_from("folder_a/folder_aa/").unwrap())
					.token(token)
					.item(crate::Item::Document {
						etag: None,
						last_modified: None,
						content: Some("DOCUMENT_CONTENT".into()),
						content_type: Some("DOCUMENT_CONTENT_TYPE".into())
					})
			)
			.await
			.status,
		crate::ResponseStatus::NotSuitableForFolderItem
	);
}

#[tokio::test]
async fn put_folder_item() {
	let mut database = Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:rw")
		.unwrap();

	assert_eq!(
		database
			.perform(
				crate::Request::put(crate::ItemPath::try_from("folder_a/folder_aa").unwrap())
					.token(token)
					.item(crate::Item::Folder {
						etag: None,
						last_modified: None,
					})
			)
			.await
			.status,
		crate::ResponseStatus::NotSuitableForFolderItem
	);
}

#[tokio::test]
async fn put_none_item() {
	let mut database = Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:rw")
		.unwrap();

	assert_eq!(
		database
			.perform(
				crate::Request::put(crate::ItemPath::try_from("folder_a/document.txt").unwrap())
					.token(token)
			)
			.await
			.status,
		crate::ResponseStatus::MissingItem
	);
}

#[tokio::test]
async fn get_not_found() {
	let mut database = Database::new(<EmptyEngineForTests as crate::Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:r")
		.unwrap();

	assert_eq!(
		database
			.perform(
				crate::Request::get(
					crate::ItemPath::try_from("folder_a/not_existing.json").unwrap()
				)
				.token(token)
				.item(crate::Item::Document {
					etag: None,
					last_modified: None,
					content: Some("DOCUMENT_CONTENT".into()),
					content_type: Some("DOCUMENT_CONTENT_TYPE".into())
				})
			)
			.await
			.status,
		crate::ResponseStatus::Performed(crate::EngineResponse::NotFound)
	);
}

impl<E: crate::Engine> Database<E> {
	pub async fn perform(&mut self, request: impl Into<crate::Request>) -> crate::Response {
		let request = request.into();

		let status = match self.is_allowed(&request) {
			Ok(()) => {
				if (request.method == crate::Method::Put || request.method == crate::Method::Delete)
					&& request.path.is_folder()
				{
					return crate::Response {
						request,
						status: crate::ResponseStatus::NotSuitableForFolderItem,
					};
				}

				let get_request = crate::Request {
					method: if request.method == crate::Method::Put {
						crate::Method::Get
					} else {
						crate::Method::Head
					},
					path: request.path.clone(),
					item: None,
					limits: vec![],
					token: None,
				};
				let get_response = self.engine.perform(&get_request).await;

				if let crate::EngineResponse::NotFound = get_response {
					if request.method == crate::Method::Put {
						return crate::Response {
							request,
							status: crate::ResponseStatus::Performed(
								crate::EngineResponse::NotFound,
							),
						};
					}
				} else if let crate::EngineResponse::GetSuccessDocument(get_document) =
					&get_response
				{
					let get_document_etag = get_document.get_etag();
					if let Some(get_document_etag) = get_document_etag {
						for limit in &request.limits {
							match limit {
								crate::Limit::IfMatch(if_match_etag) => {
									if get_document_etag != *if_match_etag {
										return crate::Response {
											request,
											status: crate::ResponseStatus::NoIfMatch(
												get_document_etag,
											),
										};
									}
								}
								crate::Limit::IfNoneMatch(none_match) => {
									if get_document_etag == *none_match
										|| *none_match == crate::Etag::from("*")
									{
										return crate::Response {
											request,
											status: crate::ResponseStatus::IfNoneMatch(
												get_document_etag,
											),
										};
									}
								}
							}
						}
					} else {
						return crate::Response {
							request,
							status: crate::ResponseStatus::InternalError(String::from(
								"get does not returns etag",
							)),
						};
					}
				}

				if request.method == crate::Method::Head || request.method == crate::Method::Get {
					crate::ResponseStatus::Performed(get_response)
				} else {
					match &request.item {
						Some(crate::Item::Document {
							etag: _,
							last_modified: _,
							content: request_content,
							content_type: request_content_type,
						}) => {
							let prehentive_answer = if request.method == crate::Method::Put {
								match &get_response {
									crate::EngineResponse::GetSuccessDocument(
										crate::Item::Document {
											etag: _,
											last_modified: _,
											content: get_content,
											content_type: get_content_type,
										},
									) => {
										if request_content == get_content
											&& request_content_type == get_content_type
										{
											Some(crate::ResponseStatus::ContentNotChanged)
										} else {
											None
										}
									}
									crate::EngineResponse::GetSuccessDocument(
										crate::Item::Folder { .. },
									) => Some(crate::ResponseStatus::NotSuitableForFolderItem),
									crate::EngineResponse::GetSuccessFolder { .. } => {
										Some(crate::ResponseStatus::NotSuitableForFolderItem)
									}
									crate::EngineResponse::CreateSuccess(_, _) => {
										Some(crate::ResponseStatus::InternalError(format!(
											"get request unexpected response : {get_response:?}"
										)))
									}
									crate::EngineResponse::UpdateSuccess(_, _) => {
										Some(crate::ResponseStatus::InternalError(format!(
											"get request unexpected response : {get_response:?}"
										)))
									}
									crate::EngineResponse::DeleteSuccess => {
										Some(crate::ResponseStatus::InternalError(format!(
											"get request unexpected response : {get_response:?}"
										)))
									}
									crate::EngineResponse::NotFound => {
										Some(crate::ResponseStatus::Performed(
											crate::EngineResponse::NotFound,
										))
									}
									crate::EngineResponse::InternalError(error) => {
										Some(crate::ResponseStatus::InternalError(error.clone()))
									}
								}
							} else {
								None
							};

							if let Some(prehentive_answer) = prehentive_answer {
								prehentive_answer
							} else {
								let engine_response = self.engine.perform(&request).await;

								if engine_response.has_muted_database() {
									for listener in &mut self.listeners {
										if let Ok(event) =
											crate::Event::build_from(&request, &engine_response)
										{
											listener.receive(event);
										}
									}
								}

								crate::ResponseStatus::Performed(engine_response)
							}
						}
						Some(crate::Item::Folder { .. }) => {
							crate::ResponseStatus::NotSuitableForFolderItem
						}
						None => crate::ResponseStatus::MissingItem,
					}
				}
			}
			Err(error) => crate::ResponseStatus::Unallowed(error),
		};

		crate::Response { request, status }
	}
	fn is_allowed(&self, request: &crate::Request) -> Result<(), AccessError> {
		if request
			.path
			.starts_with(&crate::ItemPath::try_from("/public/").unwrap())
		{
			if request.path.is_document() {
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
	fn receive(&mut self, _event: crate::Event) -> crate::Response {
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
