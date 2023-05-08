use rand::seq::IteratorRandom;
use rand::Rng;
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as AsyncMutex;

const ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789abcdefghijklmnopqrstuvwxyz";
const DEFAULT_WORKSPACE_NAME: &str = "workspace";

const DEFAULT_ENCRYPTION_KEY: [u8; 32] = [
	96, 247, 49, 178, 165, 246, 126, 169, 201, 231, 44, 4, 253, 80, 49, 233, 248, 153, 162, 186,
	144, 108, 34, 56, 19, 105, 186, 31, 145, 151, 27, 115,
];

mod assets;
mod http_server;
mod https_server;
mod settings;
mod webfinger;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let min_log_level = if cfg!(debug_assertions) {
		simplelog::LevelFilter::Debug
	} else {
		simplelog::LevelFilter::Info
	};

	let settings_file_path = if let Some(settings_file_path) = std::env::args().nth(1) {
		let new_path = std::path::PathBuf::from(settings_file_path);

		std::fs::create_dir_all(new_path.parent().unwrap()).ok();

		new_path
	} else {
		let workspace = std::path::PathBuf::from(std::env::args().next().unwrap())
			.parent()
			.unwrap()
			.join(DEFAULT_WORKSPACE_NAME);

		std::fs::create_dir_all(&workspace).ok();

		workspace.join("settings.toml")
	};

	let settings = match std::fs::read_to_string(&settings_file_path) {
		Ok(settings_file_content) => {
			println!("using settings file `{}`", settings_file_path.display());

			toml::from_str::<settings::Settings>(&settings_file_content).unwrap()
		}
		Err(error) => {
			println!(
				"error while reading settings file `{}` : `{error}`",
				settings_file_path.display()
			);

			let new_settings = settings::Settings::default();

			match std::fs::write(
				&settings_file_path,
				toml::to_string_pretty(&new_settings).unwrap(),
			) {
				Ok(()) => {
					println!(
						"successfuly created `{}` file with default settings",
						settings_file_path.display()
					);
				}
				Err(error) => {
					println!(
						"error while creating `{}` file with default settings : {error}",
						settings_file_path.display()
					);
				}
			}

			new_settings
		}
	};

	let log_file_path = if std::path::PathBuf::from(&settings.logfile_path).is_absolute() {
		std::path::PathBuf::from(&settings.logfile_path)
	} else {
		settings_file_path
			.parent()
			.unwrap()
			.join(settings.logfile_path.clone())
	};

	simplelog::CombinedLogger::init(vec![
		simplelog::TermLogger::new(
			min_log_level,
			simplelog::Config::default(),
			simplelog::TerminalMode::Mixed,
			simplelog::ColorChoice::Auto,
		),
		simplelog::WriteLogger::new(
			min_log_level,
			simplelog::Config::default(),
			std::fs::File::options()
				.create(true)
				.append(true)
				.open(log_file_path)
				.unwrap(),
		),
	])
	.unwrap();

	log::info!(
		"{} V{}",
		env!("CARGO_PKG_NAME").to_uppercase(),
		env!("CARGO_PKG_VERSION")
	);

	let data_path = if std::path::PathBuf::from(&settings.data_path).is_absolute() {
		std::path::PathBuf::from(&settings.data_path)
	} else {
		settings_file_path
			.parent()
			.unwrap()
			.join(settings.data_path.clone())
	};
	let mut storage_db = pontus_onyx::Database::new(
		<pontus_onyx_engine_filesystem::FileSystemEngine as pontus_onyx::Engine>::new(
			pontus_onyx_engine_filesystem::EngineSettings {
				path: std::path::PathBuf::from(data_path),
			},
		),
	);

	let userfile_path = if std::path::PathBuf::from(&settings.userfile_path).is_absolute() {
		std::path::PathBuf::from(&settings.userfile_path)
	} else {
		settings_file_path
			.parent()
			.unwrap()
			.join(settings.userfile_path.clone())
	};

	let encryption_key = if let Some(key) = &settings.custom_encryption_key {
		key.clone()
	} else {
		DEFAULT_ENCRYPTION_KEY
	};

	storage_db.enable_save_user(Some(userfile_path.clone()), Some(encryption_key));
	let mut has_users = false;
	if userfile_path.exists() {
		match std::fs::read(&userfile_path) {
			Ok(userfile_content) => {
				if let Ok(users) =
					serde_json::from_slice::<Vec<pontus_onyx::User>>(&userfile_content)
				{
					if !users.is_empty() {
						has_users = true;
					}

					storage_db.force_load_users(users);
				} else if let Ok(encrypted) =
					serde_json::from_slice::<Vec<Vec<u8>>>(&userfile_content)
				{
					if !encrypted.is_empty() {
						has_users = true;
					}

					let decrypted = encrypted.into_iter().map(|el| {
						match serde_encrypt::EncryptedMessage::deserialize(el) {
							Ok(el) => {
								<pontus_onyx::User as serde_encrypt::traits::SerdeEncryptSharedKey>::decrypt_owned(&el, &serde_encrypt::shared_key::SharedKey::new(encryption_key))
							},
							Err(err) => {
								Err(err)
							}
						}
					});

					if decrypted.clone().all(|el| el.is_ok()) {
						let temp = decrypted.map(|el| el.unwrap()).collect();
						storage_db.force_load_users(temp);
					} else {
						todo!()
					}
				} else {
					log::warn!("unknown data format while reading users file");
				}
			}
			Err(err) => {
				log::warn!("can not read users file : {err}");
			}
		}
	}

	if cfg!(debug_assertions) && !has_users {
		let user = {
			let mut user = String::new();
			let mut rng_limit = rand::thread_rng();
			for _ in 1..rng_limit.gen_range(16..32) {
				let mut rng_item = rand::thread_rng();
				user.push(ALPHABET.chars().choose(&mut rng_item).unwrap());
			}
			user
		};

		let mut password = {
			let mut password = String::new();
			let mut rng_limit = rand::thread_rng();
			for _ in 1..rng_limit.gen_range(16..32) {
				let mut rng_item = rand::thread_rng();
				password.push(ALPHABET.chars().choose(&mut rng_item).unwrap());
			}
			password
		};

		storage_db.create_user(&user, &mut password);
		let token = storage_db
			.generate_token(&user, &mut password, "*:rw")
			.unwrap();

		log::debug!("debug admin user : {}", user);
		log::debug!("debug admin password : {}", password);
		log::debug!("debug admin token : Bearer {}", token.0);
	}

	let storage_db = Arc::new(AsyncMutex::new(storage_db));

	let program_state = ProgramState {
		http_port: settings.port.unwrap_or_else(|| {
			let mut rng_limit = rand::thread_rng();
			rng_limit.gen_range(49152..65535)
		}),
		https_port: settings.https.as_ref().map(|https_settings| {
			https_settings.port.unwrap_or_else(|| {
				let mut rng_limit = rand::thread_rng();
				rng_limit.gen_range(49152..65535)
			})
		}),
	};

	let program_state = Arc::new(Mutex::new(program_state));

	// TODO : save/restore it on disk :
	let form_tokens = Arc::new(Mutex::new(vec![]));

	let mut thread_handles = vec![];

	thread_handles.push(
		http_server::run(
			settings.clone(),
			storage_db.clone(),
			program_state.clone(),
			form_tokens.clone(),
		)
		.unwrap(),
	);

	if settings.https.is_some() {
		match https_server::run(settings, storage_db, program_state, form_tokens) {
			Ok(https_handle) => {
				thread_handles.push(https_handle);
			}
			Err(err) => {
				log::warn!("{err}");
			}
		}
	}

	while thread_handles.iter().all(|handle| !handle.is_finished()) {}

	Ok(())
}

async fn storage(
	storage_db: actix_web::web::Data<
		Arc<AsyncMutex<pontus_onyx::Database<pontus_onyx_engine_filesystem::FileSystemEngine>>>,
	>,
	request: actix_web::HttpRequest,
	payload: actix_web::web::Payload,
) -> actix_web::HttpResponse {
	let converted_request = pontus_onyx::from_actix_request(&request, &mut payload.into_inner())
		.await
		.unwrap();

	let db_response = storage_db.lock().await.perform(converted_request).await;

	log::info!("database response : {:?}", db_response.status);

	actix_web::HttpResponse::from(db_response)
}

async fn storage_options(request: actix_web::HttpRequest) -> actix_web::HttpResponse {
	// TODO : check security issue about this ?
	let all_origins = actix_web::http::header::HeaderValue::from_bytes(b"*").unwrap();
	let origin = request
		.headers()
		.get(actix_web::http::header::ORIGIN)
		.unwrap_or(&all_origins)
		.to_str()
		.unwrap();

	// TODO : build at the end of the implementation.
	let mut response = actix_web::HttpResponse::Ok();
	response.insert_header((actix_web::http::header::CACHE_CONTROL, "no-cache"));
	response.insert_header((actix_web::http::header::ACCESS_CONTROL_ALLOW_ORIGIN, origin));

	if origin != "*" {
		response.insert_header((actix_web::http::header::VARY, "Origin"));
	}

	response.insert_header((
		actix_web::http::header::ACCESS_CONTROL_ALLOW_METHODS,
		"OPTIONS, GET, HEAD, PUT, DELETE",
	));
	response.insert_header((
		actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
		"Content-Length, Content-Type, Etag, Last-Modified",
	));
	response.insert_header((
		actix_web::http::header::ACCESS_CONTROL_ALLOW_HEADERS,
		"Authorization, Content-Length, Content-Type, Origin, If-Match, If-None-Match",
	));

	response.finish()
}

#[derive(serde::Deserialize)]
pub struct OauthGetQuery {
	redirect_uri: String,
	scope: String,
	client_id: String,
	response_type: String,
	auth_result: Option<String>,
}

#[derive(serde::Serialize)]
struct OauthContext {
	app_name: String,
	app_version: String,
	username: String,
	uri_encoded_username: String,
	errors: Vec<String>,
	form_token: String,
	client: String,
	redirect_uri: String,
	response_type: String,
	scopes: Vec<pontus_onyx::security::BearerAccess>,
	raw_scopes: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FormToken {
	ip: std::net::SocketAddr,
	usage: FormTokenUsage,
	forged: time::OffsetDateTime,
	value: String,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
enum FormTokenUsage {
	Oauth,
}

async fn get_oauth(
	path_payloads: actix_web::web::Path<String>,
	query: actix_web::web::Query<OauthGetQuery>,
	request: actix_web::HttpRequest,
	form_tokens: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<Vec<FormToken>>>>,
) -> impl actix_web::Responder {
	let username = path_payloads.into_inner();

	// TODO : do not panic if input data is incorrect (especially scopes)
	let scopes = pct_str::PctString::new(&query.scope)
		.unwrap()
		.decode()
		.split(' ')
		.map(|scope_string| {
			(std::convert::TryFrom::try_from(scope_string.trim())
				as Result<
					pontus_onyx::security::BearerAccess,
					pontus_onyx::security::BearerAccessConvertError,
				>)
				.unwrap()
		})
		.collect();

	let ip = request.peer_addr().unwrap();

	let template = std::fs::read_to_string("assets/oauth.html")
		.unwrap_or_else(|_| String::from(assets::SERVER_OAUTH));

	let mut output = vec![];
	let mut rewriter = lol_html::HtmlRewriter::new(
		lol_html::Settings {
			element_content_handlers: vec![lol_html::element!("template_remove", |el| {
				el.remove();

				Ok(())
			})],
			..lol_html::Settings::default()
		},
		|c: &[u8]| output.extend_from_slice(c),
	);
	rewriter.write(template.as_bytes()).unwrap();
	rewriter.end().unwrap();
	let template = String::from_utf8(output).unwrap();

	let mut engine = tera::Tera::default();
	engine.add_raw_template("oauth.html", &template).unwrap();

	let mut new_form_token = String::new();
	let mut rng_limit = rand::thread_rng();
	for _ in 1..rng_limit.gen_range(16..32) {
		let mut rng_item = rand::thread_rng();
		new_form_token.push(ALPHABET.chars().choose(&mut rng_item).unwrap());
	}

	let updated = match form_tokens
		.lock()
		.unwrap()
		.iter_mut()
		.find(|token| token.ip == ip && token.usage == FormTokenUsage::Oauth)
	{
		Some(token) => {
			token.forged = time::OffsetDateTime::now_utc();
			token.value = new_form_token.clone();

			true
		}
		None => false,
	};

	if !updated {
		form_tokens.lock().unwrap().push(FormToken {
			ip,
			usage: FormTokenUsage::Oauth,
			forged: time::OffsetDateTime::now_utc(),
			value: new_form_token.clone(),
		});
	}

	let context = OauthContext {
		app_name: env!("CARGO_PKG_NAME").into(),
		app_version: env!("CARGO_PKG_VERSION").into(),
		username: pct_str::PctString::new(&username)
			.unwrap()
			.decode()
			.chars()
			.collect(),
		uri_encoded_username: pct_str::PctString::encode(
			pct_str::PctString::new(&username).unwrap().decode().chars(),
			pct_str::URIReserved,
		)
		.to_string(),
		errors: if let Some(auth_result) = &query.auth_result {
			if auth_result.trim() == "" {
				vec![]
			} else {
				vec![if auth_result == "wrong_credentials" {
					String::from("Wrong credentials.")
				} else if auth_result == "security_issue" {
					String::from("There is an security issue, please try again. If error persists, please contact your administator with `security_issue` code.")
				} else {
					String::from("Unknown error.")
				}]
			}
		} else {
			vec![]
		},
		form_token: new_form_token,
		client: query.client_id.clone(),
		redirect_uri: pct_str::PctString::encode(
			pct_str::PctString::new(&query.redirect_uri)
				.unwrap()
				.decode()
				.chars(),
			pct_str::URIReserved,
		)
		.to_string(),
		response_type: query.response_type.clone(),
		scopes,
		raw_scopes: pct_str::PctString::encode(query.scope.chars(), pct_str::URIReserved)
			.to_string(),
	};

	let rendered = engine
		.render(
			"oauth.html",
			&tera::Context::from_serialize(&context).unwrap(),
		)
		.unwrap();

	actix_web::HttpResponse::Ok().body(rendered)
}

#[derive(serde::Deserialize)]
pub struct OauthPostQuery {
	redirect_uri: String,
	scopes: String,
	client_id: String,
	response_type: String,
	username: String,
	password: String,
	allow: String,
	form_token: String,
}

async fn post_oauth(
	storage_db: actix_web::web::Data<
		Arc<AsyncMutex<pontus_onyx::Database<pontus_onyx_engine_filesystem::FileSystemEngine>>>,
	>,
	request: actix_web::HttpRequest,
	form: actix_web::web::Form<OauthPostQuery>,
	form_tokens: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<Vec<FormToken>>>>,
	settings: actix_web::web::Data<crate::settings::Settings>,
	program_state: actix_web::web::Data<Arc<Mutex<ProgramState>>>,
) -> impl actix_web::Responder {
	let origin = request.headers().get("origin");

	match origin {
		Some(origin) => {
			let form_token = pct_str::PctString::new(&form.form_token).unwrap().decode();

			let form_token_found = form_tokens.lock().unwrap().iter_mut().any(|token| {
				token.usage == FormTokenUsage::Oauth
					&& token.value == form_token
					&& (time::OffsetDateTime::now_utc() - token.forged)
						<= time::Duration::minutes(5)
			});

			if form_token_found {
				let localhost = String::from("127.0.0.1");
				let current_domain = settings.domain.as_ref().unwrap_or(&localhost).clone();

				let mut allowed_domains = vec![];
				allowed_domains.push(format!(
					"http://{current_domain}:{}",
					program_state.lock().unwrap().http_port
				));
				if program_state.lock().unwrap().http_port == 80 {
					allowed_domains.push(format!("http://{current_domain}"));
				}
				if let Some(https_port) = program_state.lock().unwrap().https_port {
					allowed_domains.push(format!("https://{current_domain}:{}", https_port));
					if https_port == 443 {
						allowed_domains.push(format!("https://{current_domain}"));
					}
				}

				if allowed_domains.contains(&String::from(origin.to_str().unwrap_or_default())) {
					if form.allow == "Allow" {
						let mut password = form.password.clone();
						let token = storage_db.lock().await.generate_token(
							form.username.clone(),
							&mut password,
							pct_str::PctString::new(&form.scopes).unwrap().decode(),
						);

						match token {
							Ok(new_token) => {
								let new_path = format!(
									"{}#access_token={}&token_type={}",
									pct_str::PctString::new(&form.redirect_uri)
										.unwrap()
										.decode(),
									pct_str::PctString::encode(
										new_token.0.chars(),
										pct_str::URIReserved
									),
									"bearer"
								);

								actix_web::HttpResponse::Found()
									.insert_header((
										actix_web::http::header::LOCATION,
										new_path.clone(),
									))
									.body(format!(
										r#"Redirecting to <a href="{new_path}">{new_path}</a>"#
									))
							}
							Err(err) => {
								log::error!("error while authenticate : {err:?}");

								let new_path = format!(
									"{}?redirect_uri={}&scope={}&client_id={}&response_type={}&auth_result={}",
									form.username,
									pct_str::PctString::encode(
										pct_str::PctString::new(
											&form.redirect_uri
										)
											.unwrap()
											.decode()
											.chars(),
										pct_str::URIReserved
									),
									form.scopes,
									form.client_id,
									form.response_type,
									"wrong_credentials"
								);

								return actix_web::HttpResponse::Found()
									.insert_header((
										actix_web::http::header::LOCATION,
										new_path.clone(),
									))
									.body(format!(
										r#"Redirecting to <a href="{new_path}">{new_path}</a>"#
									));
							}
						}
					} else {
						log::error!("`allow` form field not provided");

						let new_path = format!(
							"/oauth/{}?redirect_uri={}&scope={}&client_id={}&response_type={}&auth_result={}",
							form.username,
							pct_str::PctString::encode(
								pct_str::PctString::new(
									&form.redirect_uri
								)
									.unwrap()
									.decode()
									.chars(),
								pct_str::URIReserved
							),
							form.scopes,
							form.client_id,
							form.response_type,
							"security_issue"
						);

						return actix_web::HttpResponse::Found()
							.insert_header((actix_web::http::header::LOCATION, new_path.clone()))
							.body(format!(
								r#"Redirecting to <a href="{new_path}">{new_path}</a>"#
							));
					}
				} else {
					log::error!("wrong origin : {origin:?}");

					let new_path = format!(
						"/oauth/{}?redirect_uri={}&scope={}&client_id={}&response_type={}&auth_result={}",
						form.username,
						pct_str::PctString::encode(
							pct_str::PctString::new(
								&form.redirect_uri
							)
								.unwrap()
								.decode()
								.chars(),
							pct_str::URIReserved
						),
						form.scopes,
						form.client_id,
						form.response_type,
						"security_issue"
					);

					return actix_web::HttpResponse::Found()
						.insert_header((actix_web::http::header::LOCATION, new_path.clone()))
						.body(format!(
							r#"Redirecting to <a href="{new_path}">{new_path}</a>"#
						));
				}
			} else {
				log::error!("given form token was not found in server storage");

				let new_path = format!(
					"{}?redirect_uri={}&scope={}&client_id={}&response_type={}&auth_result={}",
					form.username,
					pct_str::PctString::encode(
						pct_str::PctString::new(&form.redirect_uri)
							.unwrap()
							.decode()
							.chars(),
						pct_str::URIReserved
					),
					form.scopes,
					form.client_id,
					form.response_type,
					"security_issue"
				);

				return actix_web::HttpResponse::Found()
					.insert_header((actix_web::http::header::LOCATION, new_path.clone()))
					.body(format!(
						r#"Redirecting to <a href="{new_path}">{new_path}</a>"#
					));
			}
		}
		None => {
			log::error!("no origin given in request");

			let new_path = format!(
				"{}?redirect_uri={}&scope={}&client_id={}&response_type={}&auth_result={}",
				form.username,
				pct_str::PctString::encode(
					pct_str::PctString::new(&form.redirect_uri)
						.unwrap()
						.decode()
						.chars(),
					pct_str::URIReserved
				),
				form.scopes,
				form.client_id,
				form.response_type,
				"security_issue"
			);

			return actix_web::HttpResponse::Found()
				.insert_header((actix_web::http::header::LOCATION, new_path.clone()))
				.body(format!(
					r#"Redirecting to <a href="{new_path}">{new_path}</a>"#
				));
		}
	}
}

#[derive(serde::Serialize)]
struct IndexContext {
	app_name: String,
	app_version: String,
}

#[actix_web::get("/")]
pub async fn index() -> impl actix_web::Responder {
	let template = std::fs::read_to_string("assets/index.html")
		.unwrap_or_else(|_| String::from(assets::SERVER_INDEX));

	let mut output = vec![];
	let mut rewriter = lol_html::HtmlRewriter::new(
		lol_html::Settings {
			element_content_handlers: vec![lol_html::element!("template_remove", |el| {
				el.remove();

				Ok(())
			})],
			..lol_html::Settings::default()
		},
		|c: &[u8]| output.extend_from_slice(c),
	);
	rewriter.write(template.as_bytes()).unwrap();
	rewriter.end().unwrap();
	let template = String::from_utf8(output).unwrap();

	let mut engine = tera::Tera::default();
	engine.add_raw_template("index.html", &template).unwrap();

	let context = IndexContext {
		app_name: env!("CARGO_PKG_NAME").into(),
		app_version: env!("CARGO_PKG_VERSION").into(),
	};

	let rendered = engine
		.render(
			"index.html",
			&tera::Context::from_serialize(&context).unwrap(),
		)
		.unwrap();

	actix_web::HttpResponse::Ok().body(rendered)
}

pub async fn logo() -> impl actix_web::Responder {
	let mut res = actix_web::HttpResponse::Ok();

	res.body(actix_web::web::Bytes::from_static(crate::assets::LOGO))
}

#[actix_web::get("/assets/remotestorage.svg")]
pub async fn remotestoragesvg() -> impl actix_web::Responder {
	actix_web::HttpResponse::Ok().body(actix_web::web::Bytes::from_static(
		crate::assets::REMOTE_STORAGE,
	))
}

#[derive(Debug, Clone, Default)]
pub struct ProgramState {
	pub http_port: usize,
	pub https_port: Option<usize>,
}

fn configure_server<E: pontus_onyx::Engine + 'static>(
	settings: settings::Settings,
	database: Arc<AsyncMutex<pontus_onyx::Database<E>>>,
	program_state: Arc<Mutex<ProgramState>>,
	form_tokens: Arc<Mutex<Vec<FormToken>>>,
) -> impl FnOnce(&mut actix_web::web::ServiceConfig) {
	move |config: &mut actix_web::web::ServiceConfig| {
		config
			.app_data(actix_web::web::Data::new(settings))
			.app_data(actix_web::web::Data::new(database))
			.app_data(actix_web::web::Data::new(program_state))
			.app_data(actix_web::web::Data::new(form_tokens))
			.route(
				"/storage/{path:.*}",
				actix_web::web::method(actix_web::http::Method::OPTIONS).to(storage_options),
			)
			.route("/storage/{path:.*}", actix_web::web::head().to(storage))
			.route("/storage/{path:.*}", actix_web::web::get().to(storage))
			.route("/storage/{path:.*}", actix_web::web::put().to(storage))
			.route("/storage/{path:.*}", actix_web::web::delete().to(storage))
			.service(index)
			.route("/favicon.ico", actix_web::web::get().to(logo))
			.route("/assets/logo.png", actix_web::web::get().to(logo))
			.route("/oauth/{username}", actix_web::web::get().to(get_oauth))
			.route("/oauth/{username}", actix_web::web::post().to(post_oauth))
			.service(remotestoragesvg)
			.service(webfinger::webfinger_handle);
	}
}
