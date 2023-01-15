use rand::seq::IteratorRandom;
use rand::Rng;

const ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789abcdefghijklmnopqrstuvwxyz";
const DEFAULT_WORKSPACE_NAME: &str = "workspace";

mod settings;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let min_log_level = if cfg!(debug_assertions) {
		simplelog::LevelFilter::Debug
	} else {
		simplelog::LevelFilter::Info
	};

	let settings_file_path = if let Some(settings_file_path) = std::env::args().nth(1) {
		let new_path = std::path::PathBuf::from(settings_file_path);

		std::fs::create_dir_all(&new_path.parent().unwrap()).ok();

		new_path
	} else {
		let workspace = std::path::PathBuf::from(std::env::args().nth(0).unwrap())
			.parent()
			.unwrap()
			.join(DEFAULT_WORKSPACE_NAME);

		std::fs::create_dir_all(&workspace).ok();

		let new_path = workspace.join("settings.toml");

		new_path
	};

	let settings = match std::fs::read(&settings_file_path) {
		Ok(settings_file_content) => {
			println!("using settings file `{}`", settings_file_path.display());

			toml::from_slice::<settings::Settings>(&settings_file_content)?
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
			.join(settings.logfile_path)
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

	let mut storage_db = pontus_onyx::Database::new(pontus_onyx_memory_engine::MemoryEngine::new());

	if cfg!(debug_assertions) {
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

	let storage_db = std::sync::Arc::new(std::sync::Mutex::new(storage_db));

	let host = settings.domain.unwrap_or(String::from("127.0.0.1"));
	let port = settings.port;
	log::info!("starting server http://{host}:{port}/");

	actix_web::HttpServer::new(move || {
		actix_web::App::new()
			.app_data(actix_web::web::Data::new(storage_db.clone()))
			.wrap(actix_web::middleware::Logger::default())
			.route("/storage/{path:.*}", actix_web::web::head().to(storage))
			.route("/storage/{path:.*}", actix_web::web::get().to(storage))
			.route("/storage/{path:.*}", actix_web::web::put().to(storage))
			.route("/storage/{path:.*}", actix_web::web::delete().to(storage))
	})
	.bind(format!("{host}:{port}"))?
	.run()
	.await
}

async fn storage(
	storage_db: actix_web::web::Data<
		std::sync::Arc<
			std::sync::Mutex<pontus_onyx::Database<pontus_onyx_memory_engine::MemoryEngine>>,
		>,
	>,
	request: actix_web::HttpRequest,
	payload: actix_web::web::Payload,
) -> actix_web::HttpResponse {
	let converted_request = pontus_onyx::from_actix_request(&request, &mut payload.into_inner())
		.await
		.unwrap();

	let db_response = storage_db.lock().unwrap().perform(converted_request).await;

	log::info!("database response : {:?}", db_response.status);

	actix_web::HttpResponse::from(db_response)
}
