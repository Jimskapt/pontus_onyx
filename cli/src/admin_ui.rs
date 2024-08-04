use std::{
	collections::BTreeMap,
	path::PathBuf,
	sync::{Arc, Mutex},
};
use tokio::sync::Mutex as AsyncMutex;

pub struct AdminUI<E: pontus_onyx::Engine + Send + 'static> {
	settings: crate::settings::Settings,
	database: Arc<AsyncMutex<pontus_onyx::Database<E>>>,
	program_state: Arc<Mutex<crate::ProgramState>>,
	form_tokens: Arc<Mutex<Vec<crate::FormToken>>>,
}

impl<E: pontus_onyx::Engine + Send + 'static> AdminUI<E> {
	pub fn new(
		settings: crate::settings::Settings,
		database: Arc<AsyncMutex<pontus_onyx::Database<E>>>,
		program_state: Arc<Mutex<crate::ProgramState>>,
		form_tokens: Arc<Mutex<Vec<crate::FormToken>>>,
	) -> Self {
		Self {
			settings,
			database,
			program_state,
			form_tokens,
		}
	}

	pub fn run(self) -> Result<std::thread::JoinHandle<Result<(), std::io::Error>>, String> {
		let addr = self.get_addr();

		let bind = actix_web::HttpServer::new(move || {
			actix_web::App::new()
				.wrap(actix_web::middleware::Logger::default())
				.app_data(actix_web::web::Data::new(self.settings.clone()))
				.app_data(actix_web::web::Data::new(self.database.clone()))
				.app_data(actix_web::web::Data::new(self.program_state.clone()))
				.app_data(actix_web::web::Data::new(self.form_tokens.clone()))
				.service(get_users)
				.service(get_user)
				.service(post_user)
				.service(get_settings)
				.service(index)
		})
		.bind(addr.clone());

		match bind {
			Ok(bind) => {
				let run = bind.run();

				Ok(std::thread::spawn(move || {
					let sys = actix_web::rt::System::new();
					sys.block_on(run)
				}))
			}
			Err(err) => Err(format!("can start admin ui server : {err}")),
		}
	}

	pub fn get_addr(&self) -> String {
		let host = String::from("127.0.0.1");
		let port = self.program_state.lock().unwrap().admin_ui_port;

		format!("{host}:{port}")
	}
}

#[derive(serde::Serialize)]
struct AdminUiContext {
	app_name: String,
	app_version: String,
}

#[derive(serde::Serialize)]
struct AdminUiIndexContext {
	program: AdminUiContext,
}

#[actix_web::get("/")]
pub async fn index() -> impl actix_web::Responder {
	let template = std::fs::read_to_string("assets/admin/index.html")
		.unwrap_or_else(|_| String::from(crate::assets::ADMIN_UI_INDEX));

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

	let context = AdminUiIndexContext {
		program: AdminUiContext {
			app_name: env!("CARGO_PKG_NAME").into(),
			app_version: env!("CARGO_PKG_VERSION").into(),
		},
	};

	let rendered = engine
		.render(
			"index.html",
			&tera::Context::from_serialize(context).unwrap(),
		)
		.unwrap();

	actix_web::HttpResponse::Ok().body(rendered)
}

#[derive(serde::Serialize)]
struct AdminUiUsersContext {
	program: AdminUiContext,
	users: Vec<String>,
}

#[actix_web::get("/users")]
pub async fn get_users(
	database: actix_web::web::Data<
		Arc<AsyncMutex<pontus_onyx::Database<pontus_onyx_engine_filesystem::FileSystemEngine>>>,
	>,
) -> impl actix_web::Responder {
	let template = std::fs::read_to_string("assets/admin/users.html")
		.unwrap_or_else(|_| String::from(crate::assets::ADMIN_UI_USERS));

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
	engine.add_raw_template("users.html", &template).unwrap();

	let users = database.lock().await.get_users_list();

	let context = AdminUiUsersContext {
		program: AdminUiContext {
			app_name: env!("CARGO_PKG_NAME").into(),
			app_version: env!("CARGO_PKG_VERSION").into(),
		},
		users,
	};

	let rendered = engine
		.render(
			"users.html",
			&tera::Context::from_serialize(context).unwrap(),
		)
		.unwrap();

	actix_web::HttpResponse::Ok().body(rendered)
}

#[derive(serde::Serialize)]
struct AdminUiUserContext {
	program: AdminUiContext,
	username: Option<String>,
	user_roles: Vec<pontus_onyx::user::UserAdminRole>,
	all_available_roles: BTreeMap<pontus_onyx::user::UserAdminRole, String>,
	form_token: String,
	send_result: Vec<String>,
}

#[derive(serde::Deserialize)]
pub struct AdminUiUserQueryInfo {
	username: Option<String>,
	send_result: Option<String>,
}

#[actix_web::get("/user")]
pub async fn get_user(
	query: actix_web::web::Query<AdminUiUserQueryInfo>,
	form_tokens: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<Vec<crate::FormToken>>>>,
	database: actix_web::web::Data<
		Arc<AsyncMutex<pontus_onyx::Database<pontus_onyx_engine_filesystem::FileSystemEngine>>>,
	>,
	request: actix_web::HttpRequest,
) -> impl actix_web::Responder {
	let username = query.username.clone();

	let mut all_available_roles = BTreeMap::new();
	for role in pontus_onyx::user::ALL_ROLES {
		all_available_roles.insert(role.clone(), format!("{role}"));
	}

	let user_roles = if let Some(username) = &username {
		database
			.lock()
			.await
			.get_user_metadata(username)
			.map(|metadata| metadata.special_roles)
			.unwrap_or_else(Vec::new)
	} else {
		vec![]
	};

	let template = std::fs::read_to_string("assets/admin/user.html")
		.unwrap_or_else(|_| String::from(crate::assets::ADMIN_UI_USER));

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
	engine.add_raw_template("user.html", &template).unwrap();

	let ip = request.peer_addr().unwrap();
	let new_token = crate::FormToken::new(&ip, &crate::FormTokenUsage::UserEdit);
	let form_token = new_token.value.clone();
	form_tokens.lock().unwrap().push(new_token);

	let context = AdminUiUserContext {
		program: AdminUiContext {
			app_name: env!("CARGO_PKG_NAME").into(),
			app_version: env!("CARGO_PKG_VERSION").into(),
		},
		username,
		form_token,
		user_roles,
		all_available_roles,
		send_result: if let Some(send_result) = &query.send_result {
			if send_result.trim() == "" {
				vec![]
			} else {
				vec![if send_result == "edit_ok" {
					String::from("This user has been saved successfully.")
				} else if send_result == "delete_ok" {
					String::from("This user has been successfully deleted.")
				} else if send_result == "policy_blocking" {
					String::from("An security policy refused this new data (see logs for details).")
				} else if send_result == "security_issue" {
					String::from("There is an security issue, please try again. If error persists, please see logs and/or contact your administator with `security_issue` code.")
				} else {
					String::from("Unknown error.")
				}]
			}
		} else {
			vec![]
		},
	};

	let rendered = engine
		.render(
			"user.html",
			&tera::Context::from_serialize(context).unwrap(),
		)
		.unwrap();

	actix_web::HttpResponse::Ok().body(rendered)
}

#[derive(serde::Deserialize)]
pub struct UserFormData {
	new_username: String,
	new_password: String,
	new_roles: pontus_onyx::user::UserAdminRole,
	form_token: String,
}

#[actix_web::post("/user")]
pub async fn post_user(
	query: actix_web::web::Query<AdminUiUserQueryInfo>,
	form_tokens: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<Vec<crate::FormToken>>>>,
	form_data: actix_web::web::Form<UserFormData>,
	database: actix_web::web::Data<
		Arc<AsyncMutex<pontus_onyx::Database<pontus_onyx_engine_filesystem::FileSystemEngine>>>,
	>,
) -> impl actix_web::Responder {
	let old_username = query.username.clone();

	let new_username = form_data.new_username.clone();
	let new_password = form_data.new_password.clone();
	let new_roles = form_data.new_roles.clone();
	let form_token = form_data.form_token.clone();

	let form_token_found = form_tokens.lock().unwrap().iter_mut().any(|token| {
		token.usage == crate::FormTokenUsage::UserEdit
			&& token.value == form_token
			&& (time::OffsetDateTime::now_utc() - token.forged) <= time::Duration::minutes(5)
	});

	let new_path = if form_token_found {
		let mut db = database.lock().await;

		let (remove, mut new_password) = if let Some(old_username) = &old_username {
			match db.remove_user(old_username.clone()) {
				Ok(old_password) => {
					if new_password.is_empty() {
						(true, old_password)
					} else {
						(true, zeroize::Zeroizing::new(new_password))
					}
				}
				Err(err) => {
					log::warn!("remove user error : {err}");

					(false, zeroize::Zeroizing::new(String::from("")))
				}
			}
		} else {
			(true, zeroize::Zeroizing::new(new_password))
		};

		if remove {
			if !new_username.is_empty() {
				match db.create_user(new_username.clone(), &mut new_password, &[new_roles]) {
					Ok(()) => {
						format!("/user?username={new_username}&send_result=edit_ok",)
					}
					Err(err) => {
						log::warn!("save user error : {err}");

						let error_code = if err.starts_with("policy `") {
							"policy_blocking"
						} else {
							"edit_error"
						};

						if let Some(old_username) = old_username {
							format!("/user?username={old_username}&send_result={error_code}",)
						} else {
							format!("/user?send_result={error_code}",)
						}
					}
				}
			} else {
				String::from("/user?send_result=delete_ok")
			}
		} else if let Some(old_username) = old_username {
			format!("/user?username={old_username}&send_result=edit_error",)
		} else {
			String::from("/user?send_result=edit_error")
		}
	} else {
		log::error!("given form token was not found in server storage");

		if let Some(old_username) = old_username {
			format!("/user?username={old_username}&send_result=security_issue",)
		} else {
			String::from("/user?send_result=security_issue")
		}
	};

	return actix_web::HttpResponse::Found()
		.insert_header((actix_web::http::header::LOCATION, new_path.clone()))
		.body(format!(
			r#"Redirecting to <a href="{new_path}">{new_path}</a>"#
		));
}

#[derive(serde::Serialize)]
struct AdminUiSettingsContext {
	program: AdminUiContext,
	settings_path: PathBuf,
}

#[actix_web::get("/settings")]
pub async fn get_settings(
	program_state: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<crate::ProgramState>>>,
) -> impl actix_web::Responder {
	let template = std::fs::read_to_string("assets/admin/settings.html")
		.unwrap_or_else(|_| String::from(crate::assets::ADMIN_UI_SETTINGS));

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
	engine.add_raw_template("settings.html", &template).unwrap();

	let context = AdminUiSettingsContext {
		program: AdminUiContext {
			app_name: env!("CARGO_PKG_NAME").into(),
			app_version: env!("CARGO_PKG_VERSION").into(),
		},
		settings_path: program_state.lock().unwrap().settings_path.clone(),
	};

	let rendered = engine
		.render(
			"settings.html",
			&tera::Context::from_serialize(context).unwrap(),
		)
		.unwrap();

	actix_web::HttpResponse::Ok().body(rendered)
}
