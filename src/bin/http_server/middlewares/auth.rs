use rand::seq::IteratorRandom;
use rand::Rng;
use std::sync::{Arc, Mutex};

pub struct Auth {
	pub logger: Arc<Mutex<charlie_buffalo::Logger>>,
}

impl<S> actix_web::dev::Transform<S> for Auth
where
	S: actix_web::dev::Service<
		Request = actix_web::dev::ServiceRequest,
		Response = actix_web::dev::ServiceResponse<actix_web::dev::Body>,
		Error = actix_web::Error,
	>,
	S::Future: 'static,
{
	type Request = actix_web::dev::ServiceRequest;
	type Response = actix_web::dev::ServiceResponse<actix_web::dev::Body>;
	type Error = actix_web::Error;
	type InitError = ();
	type Transform = AuthMiddleware<S>;
	type Future = futures::future::Ready<Result<Self::Transform, Self::InitError>>;

	fn new_transform(&self, service: S) -> Self::Future {
		futures::future::ok(Self::Transform {
			service,
			logger: self.logger.clone(),
		})
	}
}

pub struct AuthMiddleware<S> {
	service: S,
	logger: Arc<Mutex<charlie_buffalo::Logger>>,
}

type OutputFuture = std::pin::Pin<
	Box<
		dyn futures::Future<
			Output = Result<
				actix_web::dev::ServiceResponse<actix_web::dev::Body>,
				actix_web::Error,
			>,
		>,
	>,
>;

impl<S> actix_web::dev::Service for AuthMiddleware<S>
where
	S: actix_web::dev::Service<
		Request = actix_web::dev::ServiceRequest,
		Response = actix_web::dev::ServiceResponse<actix_web::dev::Body>,
		Error = actix_web::Error,
	>,
	S::Future: 'static,
{
	type Request = actix_web::dev::ServiceRequest;
	type Response = actix_web::dev::ServiceResponse<actix_web::dev::Body>;
	type Error = actix_web::Error;
	type Future = OutputFuture;

	fn poll_ready(
		&mut self,
		ctx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Result<(), Self::Error>> {
		self.service.poll_ready(ctx)
	}

	fn call(&mut self, service_request: Self::Request) -> Self::Future {
		let request_method = service_request.method().clone();

		match service_request.head().headers().get("Authorization") {
			Some(auth_value) => {
				let search_token = auth_value
					.to_str()
					.unwrap()
					.strip_prefix("Bearer ")
					.unwrap()
					.trim();
				let tokens = service_request
					.app_data::<actix_web::web::Data<
						std::sync::Arc<std::sync::Mutex<Vec<crate::http_server::AccessBearer>>>,
					>>()
					.unwrap()
					.lock()
					.unwrap()
					.clone();

				let settings = service_request
					.app_data::<actix_web::web::Data<
						std::sync::Arc<std::sync::Mutex<crate::http_server::Settings>>,
					>>()
					.unwrap()
					.lock()
					.unwrap()
					.clone();

				match tokens.iter().find(|e| e.get_name() == search_token) {
					Some(token) => {
						// TODO : check token validity with client_id
						// TODO : test access in case of module = "*"
						// TODO : test token expiration

						if (std::time::Instant::now() - *token.get_emit_time())
							< std::time::Duration::from_secs(settings.token_lifetime_seconds)
						{
							match token.get_scopes().iter().find(|scope| {
								if scope
									.allowed_methods()
									.iter()
									.any(|method| method == service_request.method())
								{
									if scope.module == "*" {
										service_request.path().starts_with("/storage/")
									} else {
										service_request.path().starts_with(&format!(
											"/storage/{}/{}",
											token.get_username(),
											scope.module
										)) || service_request.path().starts_with(&format!(
											"/storage/public/{}/{}",
											token.get_username(),
											scope.module
										))
									}
								} else {
									false
								}
							}) {
								Some(_) => {
									let future = self.service.call(service_request);
									Box::pin(async move { future.await })
								}
								None => {
									self.logger.lock().unwrap().push(
										vec![
											(String::from("event"), String::from("auth")),
											(String::from("token"), String::from(search_token)),
											(String::from("level"), String::from("DEBUG")),
										],
										Some("unauthorized scope"),
									);

									Box::pin(async move {
										Ok(actix_web::dev::ServiceResponse::new(
											service_request.into_parts().0,
											pontus_onyx::database::build_http_json_response(
												&request_method,
												actix_web::http::StatusCode::FORBIDDEN,
												None,
												None,
												true,
											),
										))
									})
								}
							}
						} else {
							self.logger.lock().unwrap().push(
								vec![
									(String::from("event"), String::from("auth")),
									(String::from("token"), String::from(search_token)),
									(String::from("level"), String::from("DEBUG")),
								],
								Some("expirated token"),
							);

							Box::pin(async move {
								Ok(actix_web::dev::ServiceResponse::new(
									service_request.into_parts().0,
									pontus_onyx::database::build_http_json_response(
										&request_method,
										actix_web::http::StatusCode::FORBIDDEN,
										None,
										None,
										true,
									),
								))
							})
						}
					}
					None => Box::pin(async move {
						Ok(actix_web::dev::ServiceResponse::new(
							service_request.into_parts().0,
							pontus_onyx::database::build_http_json_response(
								&request_method,
								actix_web::http::StatusCode::UNAUTHORIZED,
								None,
								None,
								true,
							),
						))
					}),
				}
			}
			None => {
				if service_request.path().starts_with("/storage/public/")
					&& service_request
						.path()
						.split('/')
						.collect::<Vec<&str>>()
						.last()
						.unwrap_or(&"") != &""
					&& (service_request.method() == actix_web::http::Method::GET
						|| service_request.method() == actix_web::http::Method::HEAD
						|| service_request.method() == actix_web::http::Method::OPTIONS)
				{
					let future = self.service.call(service_request);
					Box::pin(async move { future.await })
				} else if service_request.path().starts_with("/.well-known/webfinger")
					&& (service_request.method() == actix_web::http::Method::GET
						|| service_request.method() == actix_web::http::Method::HEAD
						|| service_request.method() == actix_web::http::Method::OPTIONS)
				{
					let future = self.service.call(service_request);
					Box::pin(async move { future.await })
				} else if service_request.path().starts_with("/oauth")
					&& (service_request.method() == actix_web::http::Method::GET
						|| service_request.method() == actix_web::http::Method::POST)
				{
					let future = self.service.call(service_request);
					Box::pin(async move { future.await })
				} else if service_request.path() == "/favicon.ico"
					&& service_request.method() == actix_web::http::Method::GET
				{
					let future = self.service.call(service_request);
					Box::pin(async move { future.await })
				} else if service_request.path() == "/remotestorage.svg"
					&& service_request.method() == actix_web::http::Method::GET
				{
					let future = self.service.call(service_request);
					Box::pin(async move { future.await })
				} else if service_request.path() == "/"
					&& service_request.method() == actix_web::http::Method::GET
				{
					let future = self.service.call(service_request);
					Box::pin(async move { future.await })
				} else if service_request.method() == actix_web::http::Method::OPTIONS {
					let future = self.service.call(service_request);
					Box::pin(async move { future.await })
				} else {
					Box::pin(async move {
						Ok(actix_web::dev::ServiceResponse::new(
							service_request.into_parts().0,
							pontus_onyx::database::build_http_json_response(
								&request_method,
								actix_web::http::StatusCode::UNAUTHORIZED,
								None,
								None,
								true,
							),
						))
					})
				}
			}
		}
	}
}

#[derive(Clone, Debug)]
pub struct OauthFormToken {
	ip: std::net::SocketAddr,
	forged: std::time::Instant,
	value: String,
}
impl OauthFormToken {
	pub fn new(ip: std::net::SocketAddr) -> Self {
		let forged = std::time::Instant::now();
		let mut value = String::new();

		let mut rng_limit = rand::thread_rng();
		for _ in 1..rng_limit.gen_range(32..64) {
			let mut rng_item = rand::thread_rng();
			value.push(
				crate::http_server::FORM_TOKEN_ALPHABET
					.chars()
					.choose(&mut rng_item)
					.unwrap(),
			);
		}

		Self { ip, forged, value }
	}
}
impl OauthFormToken {
	/*
	pub fn get_ip(&self) -> &std::net::SocketAddr {
		&self.ip
	}
	pub fn get_forged(&self) -> &std::time::Instant {
		&self.forged
	}
	*/
	pub fn get_value(&self) -> &str {
		&self.value
	}
	pub fn has_expirated(&self) -> bool {
		(std::time::Instant::now() - self.forged) == std::time::Duration::from_secs(5 * 60)
	}
	pub fn should_be_cleaned(&self, ip: &std::net::SocketAddr) -> bool {
		if self.has_expirated() {
			return true;
		}

		if &self.ip == ip {
			return true;
		}

		return false;
	}
}

#[actix_rt::test]
async fn hsv5femo2qgu80gbad0ov5() {
	let settings = std::sync::Arc::new(std::sync::Mutex::new(
		crate::http_server::Settings::default(),
	));

	let logger = Arc::new(Mutex::new(charlie_buffalo::Logger::new(
		charlie_buffalo::new_dispatcher(Box::new(|_| {})),
		None,
	)));

	let mut app = actix_web::test::init_service(
		actix_web::App::new()
			.data(settings.clone())
			.wrap(super::Auth { logger })
			.service(crate::http_server::favicon)
			.service(crate::http_server::api::get_item)
			.service(crate::http_server::webfinger_handle)
			.service(crate::http_server::get_oauth)
			.service(crate::http_server::post_oauth),
	)
	.await;

	let tests = vec![
		(010, "/storage/user/", true),
		(020, "/storage/user/folder/", true),
		(030, "/storage/user/document", true),
		(040, "/storage/user/folder/document", true),
		(050, "/storage/public/user/folder/", true),
		(060, "/storage/public/user/document", false),
		(070, "/storage/public/user/folder/document", false),
		(080, "/.well-known/webfinger", false),
		(090, "/oauth", false),
		(100, "/favicon.ico", false),
		(110, "/remotestorage.svg", false),
		(120, "/", false),
	];

	for test in tests {
		print!("#{:03} : GET request to {} ... ", test.0, test.1);

		let request = actix_web::test::TestRequest::get().uri(test.1).to_request();
		let response = actix_web::test::call_service(&mut app, request).await;

		if test.2 {
			assert_eq!(response.status(), actix_web::http::StatusCode::UNAUTHORIZED);
		} else {
			assert_ne!(response.status(), actix_web::http::StatusCode::UNAUTHORIZED);
		}

		println!("OK");
	}
}

#[cfg(test)]
mod tests {
	use actix_web::HttpMessage;
	use std::sync::{Arc, Mutex};

	#[actix_rt::test]
	async fn kp6m20xdwvw6v4t3yxq() {
		let access_tokens: std::sync::Arc<std::sync::Mutex<Vec<crate::http_server::AccessBearer>>> =
			std::sync::Arc::new(std::sync::Mutex::new(vec![]));

		let token = crate::http_server::AccessBearer::new(
			vec![
				crate::http_server::Scope {
					module: String::from("folder_write"),
					right_type: crate::http_server::ScopeRightType::ReadWrite,
				},
				crate::http_server::Scope {
					module: String::from("folder_read"),
					right_type: crate::http_server::ScopeRightType::Read,
				},
			],
			"test",
			"user",
		);
		access_tokens.lock().unwrap().push(token.clone());

		let database = pontus_onyx::Database::new(
			&pontus_onyx::database::DataSource::Memory(pontus_onyx::Item::new_folder(vec![(
				"user",
				pontus_onyx::Item::new_folder(vec![
					(
						"folder_write",
						pontus_onyx::Item::new_folder(vec![(
							"a",
							pontus_onyx::Item::Document {
								etag: ulid::Ulid::new().to_string(),
								content: b"HELLO".to_vec(),
								content_type: String::from("text/plain"),
								last_modified: chrono::Utc::now(),
							},
						)]),
					),
					(
						"folder_read",
						pontus_onyx::Item::new_folder(vec![(
							"a",
							pontus_onyx::Item::Document {
								etag: ulid::Ulid::new().to_string(),
								content: b"HELLO".to_vec(),
								content_type: String::from("text/plain"),
								last_modified: chrono::Utc::now(),
							},
						)]),
					),
					(
						"public",
						pontus_onyx::Item::new_folder(vec![
							(
								"folder_write",
								pontus_onyx::Item::new_folder(vec![(
									"a",
									pontus_onyx::Item::Document {
										etag: ulid::Ulid::new().to_string(),
										content: b"HELLO".to_vec(),
										content_type: String::from("text/plain"),
										last_modified: chrono::Utc::now(),
									},
								)]),
							),
							(
								"folder_read",
								pontus_onyx::Item::new_folder(vec![(
									"a",
									pontus_onyx::Item::Document {
										etag: ulid::Ulid::new().to_string(),
										content: b"HELLO".to_vec(),
										content_type: String::from("text/plain"),
										last_modified: chrono::Utc::now(),
									},
								)]),
							),
						]),
					),
				]),
			)])),
			None,
		)
		.unwrap();
		let database = std::sync::Arc::new(std::sync::Mutex::new(database));

		let settings = std::sync::Arc::new(std::sync::Mutex::new(
			crate::http_server::Settings::default(),
		));

		let logger = Arc::new(Mutex::new(charlie_buffalo::Logger::new(
			charlie_buffalo::new_dispatcher(Box::new(|_| {})),
			None,
		)));

		let mut app = actix_web::test::init_service(
			actix_web::App::new()
				.data(database.clone())
				.data(access_tokens.clone())
				.data(settings.clone())
				.wrap(super::Auth { logger })
				.service(crate::http_server::api::get_item)
				.service(crate::http_server::api::put_item),
		)
		.await;

		let tests: Vec<(
			usize,
			actix_web::test::TestRequest,
			actix_web::http::StatusCode,
		)> = vec![
			(
				010,
				actix_web::test::TestRequest::get().uri("/storage/user/folder_read/"),
				actix_web::http::StatusCode::UNAUTHORIZED,
			),
			(
				020,
				actix_web::test::TestRequest::get().uri("/storage/user/folder_write/"),
				actix_web::http::StatusCode::UNAUTHORIZED,
			),
			(
				030,
				actix_web::test::TestRequest::get().uri("/storage/user/other/"),
				actix_web::http::StatusCode::UNAUTHORIZED,
			),
			(
				040,
				actix_web::test::TestRequest::get()
					.uri("/storage/user/folder_read/")
					.header(
						actix_web::http::header::AUTHORIZATION,
						format!("Bearer {}", token.get_name()),
					),
				actix_web::http::StatusCode::OK,
			),
			(
				050,
				actix_web::test::TestRequest::get()
					.uri("/storage/other_user/folder_read/")
					.header(
						actix_web::http::header::AUTHORIZATION,
						format!("Bearer {}", token.get_name()),
					),
				actix_web::http::StatusCode::FORBIDDEN,
			),
			(
				055,
				actix_web::test::TestRequest::get()
					.uri("/storage/user/should_not_be_accessed_by_this_token/")
					.header(
						actix_web::http::header::AUTHORIZATION,
						format!("Bearer {}", token.get_name()),
					),
				actix_web::http::StatusCode::FORBIDDEN,
			),
			(
				056,
				actix_web::test::TestRequest::get()
					.uri("/storage/user/should_not_be_accessed_by_this_token")
					.header(
						actix_web::http::header::AUTHORIZATION,
						format!("Bearer {}", token.get_name()),
					),
				actix_web::http::StatusCode::FORBIDDEN,
			),
			(
				060,
				actix_web::test::TestRequest::get()
					.uri("/storage/user/folder_write/")
					.header(
						actix_web::http::header::AUTHORIZATION,
						format!("Bearer {}", token.get_name()),
					),
				actix_web::http::StatusCode::OK,
			),
			(
				070,
				actix_web::test::TestRequest::get()
					.uri("/storage/user/should_not_be_accessed_by_this_token/")
					.header(
						actix_web::http::header::AUTHORIZATION,
						format!("Bearer {}", token.get_name()),
					),
				actix_web::http::StatusCode::FORBIDDEN,
			),
			(
				075,
				actix_web::test::TestRequest::get()
					.uri("/storage/user/should_not_be_accessed_by_this_token")
					.header(
						actix_web::http::header::AUTHORIZATION,
						format!("Bearer {}", token.get_name()),
					),
				actix_web::http::StatusCode::FORBIDDEN,
			),
			(
				080,
				actix_web::test::TestRequest::get()
					.uri("/storage/user/folder_read/")
					.header(
						actix_web::http::header::AUTHORIZATION,
						format!("Bearer {}", "RANDOM_BEARER"),
					),
				actix_web::http::StatusCode::UNAUTHORIZED,
			),
			(
				090,
				actix_web::test::TestRequest::get()
					.uri("/storage/user/folder_write/")
					.header(
						actix_web::http::header::AUTHORIZATION,
						format!("Bearer {}", "RANDOM_BEARER"),
					),
				actix_web::http::StatusCode::UNAUTHORIZED,
			),
			(
				100,
				actix_web::test::TestRequest::get()
					.uri("/storage/user/other/")
					.header(
						actix_web::http::header::AUTHORIZATION,
						format!("Bearer {}", "RANDOM_BEARER"),
					),
				actix_web::http::StatusCode::UNAUTHORIZED,
			),
			(
				110,
				actix_web::test::TestRequest::put()
					.uri("/storage/user/folder_read/b")
					.header(
						actix_web::http::header::AUTHORIZATION,
						format!("Bearer {}", token.get_name()),
					)
					.set_json(&serde_json::json!({"value": "HELLO"})),
				actix_web::http::StatusCode::FORBIDDEN,
			),
			(
				120,
				actix_web::test::TestRequest::put()
					.uri("/storage/user/folder_write/b")
					.header(
						actix_web::http::header::AUTHORIZATION,
						format!("Bearer {}", token.get_name()),
					)
					.set_json(&serde_json::json!({"value": "HELLO"})),
				actix_web::http::StatusCode::CREATED,
			),
			(
				130,
				actix_web::test::TestRequest::put()
					.uri("/storage/other_user/folder_write/b")
					.header(
						actix_web::http::header::AUTHORIZATION,
						format!("Bearer {}", token.get_name()),
					)
					.set_json(&serde_json::json!({"value": "HELLO"})),
				actix_web::http::StatusCode::FORBIDDEN,
			),
			(
				140,
				actix_web::test::TestRequest::put()
					.uri("/storage/user/other/b")
					.header(
						actix_web::http::header::AUTHORIZATION,
						format!("Bearer {}", token.get_name()),
					)
					.set_json(&serde_json::json!({"value": "HELLO"})),
				actix_web::http::StatusCode::FORBIDDEN,
			),
			(
				150,
				actix_web::test::TestRequest::put()
					.uri("/storage/public/user/folder_read/b")
					.header(
						actix_web::http::header::AUTHORIZATION,
						format!("Bearer {}", token.get_name()),
					)
					.set_json(&serde_json::json!({"value": "HELLO"})),
				actix_web::http::StatusCode::FORBIDDEN,
			),
			(
				160,
				actix_web::test::TestRequest::put()
					.uri("/storage/public/user/folder_write/b")
					.header(
						actix_web::http::header::AUTHORIZATION,
						format!("Bearer {}", token.get_name()),
					)
					.set_json(&serde_json::json!({"value": "HELLO"})),
				actix_web::http::StatusCode::CREATED,
			),
			(
				170,
				actix_web::test::TestRequest::put()
					.uri("/storage/public/user/other/b")
					.header(
						actix_web::http::header::AUTHORIZATION,
						format!("Bearer {}", token.get_name()),
					)
					.set_json(&serde_json::json!({"value": "HELLO"})),
				actix_web::http::StatusCode::FORBIDDEN,
			),
			(
				180,
				actix_web::test::TestRequest::put()
					.uri("/storage/user/folder_read/b")
					.header(
						actix_web::http::header::AUTHORIZATION,
						format!("Bearer {}", "RANDOM_BEARER"),
					)
					.set_json(&serde_json::json!({"value": "HELLO"})),
				actix_web::http::StatusCode::UNAUTHORIZED,
			),
			(
				190,
				actix_web::test::TestRequest::put()
					.uri("/storage/user/folder_write/b")
					.header(
						actix_web::http::header::AUTHORIZATION,
						format!("Bearer {}", "RANDOM_BEARER"),
					)
					.set_json(&serde_json::json!({"value": "HELLO"})),
				actix_web::http::StatusCode::UNAUTHORIZED,
			),
			(
				200,
				actix_web::test::TestRequest::put()
					.uri("/storage/user/other/b")
					.header(
						actix_web::http::header::AUTHORIZATION,
						format!("Bearer {}", "RANDOM_BEARER"),
					)
					.set_json(&serde_json::json!({"value": "HELLO"})),
				actix_web::http::StatusCode::UNAUTHORIZED,
			),
		];

		for test in tests {
			let request = test.1.to_request();
			print!(
				"#{:03} : {} request to {} with Authorization = {:?} ... ",
				test.0,
				request.method(),
				request.path(),
				match request
					.headers()
					.iter()
					.find(|&(name, _)| name == actix_web::http::header::AUTHORIZATION)
				{
					Some((_, value)) => format!("{}[...]", &value.to_str().unwrap()[7..7 + 10]),
					None => String::from("None"),
				}
			);

			let response = actix_web::test::call_service(&mut app, request).await;

			assert_eq!(response.status(), test.2);

			println!("OK");
		}
	}
}
