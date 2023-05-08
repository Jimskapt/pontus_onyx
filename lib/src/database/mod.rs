use std::collections::BTreeMap;

use rand::{seq::IteratorRandom, Rng};

use crate::{Engine, EngineResponse, Method, Response, ResponseStatus};

#[cfg(test)]
pub mod tests;

pub struct Database<E: Engine> {
	engine: E,
	users: Vec<crate::User>,
	listeners: Vec<Box<dyn crate::Listener + Send>>,
	settings: crate::DatabaseSettings,
}

impl<E: Engine> Database<E> {
	pub fn new(engine: E) -> Self {
		Self {
			engine,
			users: vec![],
			listeners: vec![],
			settings: crate::DatabaseSettings::default(),
		}
	}
	pub fn enable_save_user(
		&mut self,
		userfile_path: Option<impl Into<std::path::PathBuf>>,
		encrypt_key: Option<[u8; 32]>,
	) {
		self.settings.userfile_path = userfile_path.map(|value| value.into());
		self.settings.encryption_key = encrypt_key;
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

		self.save_on_disk();
	}
	pub fn force_load_users(&mut self, users: Vec<crate::User>) {
		self.users = users;
	}
	fn save_on_disk(&self) {
		if let Some(userfile_path) = &self.settings.userfile_path {
			if let Some(encryption_key) = &self.settings.encryption_key {
				std::fs::write(
					userfile_path,
					serde_json::to_string(
						&self
							.users
							.iter()
							.map(|user| {
								serde_encrypt::traits::SerdeEncryptSharedKey::encrypt(
									user,
									&serde_encrypt::shared_key::SharedKey::new(*encryption_key),
								)
								.unwrap() // TODO
								.serialize()
							})
							.collect::<Vec<Vec<u8>>>(),
					)
					.unwrap(), // TODO
				)
				.unwrap()
			} else {
				std::fs::write(userfile_path, serde_json::to_string(&self.users).unwrap()).unwrap()
				// TODO
			}
		}
	}
	pub fn generate_token(
		&mut self,
		username: impl Into<String>,
		password: &mut str,
		bearers: impl Into<String>,
	) -> Result<crate::security::Token, AuthenticateError> {
		let username = username.into();
		let bearers = bearers.into();

		if let Some(user) = self.users.iter_mut().find(|user| user.username == username) {
			if user.password == *password {
				let mut converted_bearers = vec![];

				for bearer in bearers.split(',') {
					match crate::security::BearerAccess::try_from(bearer) {
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
				let name = crate::security::Token::from(name);

				user.tokens.insert(
					name.clone(),
					crate::security::TokenMetadata::new(converted_bearers),
				);

				self.save_on_disk();

				Ok(name)
			} else {
				return Err(AuthenticateError::WrongPassword);
			}
		} else {
			return Err(AuthenticateError::UserNotFound(username.clone()));
		}
	}
	pub fn register_listener(&mut self, listener: Box<dyn crate::Listener + Send>) {
		self.listeners.push(listener);
	}
}

impl<E: Engine> Database<E> {
	pub async fn perform(&mut self, request: impl Into<crate::Request>) -> Response {
		let request = request.into();

		log::info!("performing {request:?}");

		let status = match self.is_allowed(&request) {
			Ok(()) => {
				if (request.method == Method::Put || request.method == Method::Delete)
					&& request.path.is_folder()
				{
					return Response {
						request,
						status: ResponseStatus::NotSuitableForFolderItem,
					};
				}

				let get_request = crate::Request {
					method: if request.method == Method::Put {
						Method::Get
					} else {
						Method::Head
					},
					path: request.path.clone(),
					item: None,
					limits: vec![],
					token: None,
					origin: crate::security::Origin::from("database_internal"),
				};
				let get_response = self.engine.perform(&get_request).await;

				log::debug!("engine item GET reponse : {get_response:?}");

				if let EngineResponse::NotFound = get_response {
					if request.method == Method::Put
						&& request.limits.iter().any(|limit| {
							if let crate::Limit::IfMatch(etag) = limit {
								*etag != crate::item::Etag::from("*")
							} else {
								false
							}
						}) {
						return Response {
							request,
							status: ResponseStatus::Performed(EngineResponse::NotFound),
						};
					}
				} else if let EngineResponse::GetSuccessDocument(get_document) = &get_response {
					let get_document_etag = get_document.get_etag();
					if let Some(get_document_etag) = get_document_etag {
						for limit in &request.limits {
							match limit {
								crate::Limit::IfMatch(if_match_etag) => {
									if get_document_etag != *if_match_etag {
										return Response {
											request,
											status: ResponseStatus::NoIfMatch(get_document_etag),
										};
									}
								}
								crate::Limit::IfNoneMatch(none_match) => {
									if get_document_etag == *none_match
										|| *none_match == crate::item::Etag::from("*")
									{
										return Response {
											request,
											status: ResponseStatus::IfNoneMatch(get_document_etag),
										};
									}
								}
							}
						}
					} else {
						return Response {
							request,
							status: ResponseStatus::InternalError(String::from(
								"get does not returns etag",
							)),
						};
					}
				}

				if request.method == Method::Head {
					ResponseStatus::Performed(get_response)
				} else {
					match &request.item {
						Some(crate::item::Item::Document {
							etag: _,
							last_modified: _,
							content: request_content,
							content_type: request_content_type,
						}) => {
							let prehentive_answer = if request.method == Method::Put {
								match &get_response {
									EngineResponse::GetSuccessDocument(
										crate::item::Item::Document {
											etag: _,
											last_modified: _,
											content: get_content,
											content_type: get_content_type,
										},
									) => {
										if request_content == get_content
											&& request_content_type == get_content_type
										{
											Some(ResponseStatus::ContentNotChanged)
										} else {
											None
										}
									}
									EngineResponse::GetSuccessDocument(
										crate::item::Item::Folder { .. },
									) => Some(ResponseStatus::NotSuitableForFolderItem),
									EngineResponse::GetSuccessFolder { .. } => {
										Some(ResponseStatus::NotSuitableForFolderItem)
									}
									EngineResponse::CreateSuccess(_, _) => {
										Some(ResponseStatus::InternalError(format!(
											"get request unexpected response : {get_response:?}"
										)))
									}
									EngineResponse::UpdateSuccess(_, _) => {
										Some(ResponseStatus::InternalError(format!(
											"get request unexpected response : {get_response:?}"
										)))
									}
									EngineResponse::DeleteSuccess => {
										Some(ResponseStatus::InternalError(format!(
											"get request unexpected response : {get_response:?}"
										)))
									}
									EngineResponse::NotFound => {
										if request.limits.iter().any(|limit| {
											if let crate::Limit::IfMatch(etag) = limit {
												*etag != crate::item::Etag::from("*")
											} else {
												false
											}
										}) {
											Some(ResponseStatus::Performed(
												EngineResponse::NotFound,
											))
										} else {
											None
										}
									}
									EngineResponse::InternalError(error) => {
										Some(ResponseStatus::InternalError(error.clone()))
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

								ResponseStatus::Performed(engine_response)
							}
						}
						Some(crate::item::Item::Folder { .. }) => {
							ResponseStatus::NotSuitableForFolderItem
						}
						None => {
							if request.method == crate::Method::Put {
								ResponseStatus::MissingRequestItem
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

								ResponseStatus::Performed(engine_response)
							}
						}
					}
				}
			}
			Err(error) => ResponseStatus::Unauthorized(error),
		};

		Response { request, status }
	}
	fn is_allowed(&self, request: &crate::Request) -> Result<(), AccessError> {
		if request
			.path
			.starts_with(&crate::item::Path::try_from("/public/").unwrap())
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
		let token = crate::security::Token::from(request.token.as_ref().unwrap().0.clone()); // already checked in calling function so unwrap is OK

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

#[derive(Debug, PartialEq)]
pub enum AccessError {
	CanNotListPublic,
	MissingToken,
	UnknownToken,
	NotValidToken(Vec<crate::security::TokenValidityError>),
}

#[derive(Debug, PartialEq)]
pub enum AuthenticateError {
	WrongBearerSyntax(String, crate::security::BearerAccessConvertError),
	UserNotFound(String),
	WrongPassword,
}
