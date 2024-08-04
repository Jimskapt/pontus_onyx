use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as AsyncMutex;

pub struct HTTPServer<T: pontus_onyx::Engine + Send + 'static> {
	settings: crate::settings::Settings,
	storage_db: Arc<AsyncMutex<pontus_onyx::Database<T>>>,
	program_state: Arc<Mutex<crate::ProgramState>>,
	form_tokens: Arc<Mutex<Vec<crate::FormToken>>>,
}

impl<T: pontus_onyx::Engine + Send + 'static> HTTPServer<T> {
	pub fn new(
		settings: crate::settings::Settings,
		storage_db: Arc<AsyncMutex<pontus_onyx::Database<T>>>,
		program_state: Arc<Mutex<crate::ProgramState>>,
		form_tokens: Arc<Mutex<Vec<crate::FormToken>>>,
	) -> Self {
		Self {
			settings,
			storage_db,
			program_state,
			form_tokens,
		}
	}

	pub fn run(self) -> Result<std::thread::JoinHandle<Result<(), std::io::Error>>, String> {
		let addr = self.get_addr();

		let bind = actix_web::HttpServer::new(move || {
			actix_web::App::new()
				.wrap(actix_web::middleware::Logger::default())
				.configure(crate::configure_server(
					self.settings.clone(),
					self.storage_db.clone(),
					self.program_state.clone(),
					self.form_tokens.clone(),
				))
		})
		.bind(addr.clone());

		match bind {
			Ok(bind) => {
				log::info!("starting unsafe data server at http://{addr}");

				let run = bind.run();

				Ok(std::thread::spawn(move || {
					let sys = actix_web::rt::System::new();
					sys.block_on(run)
				}))
			}
			Err(err) => Err(format!("can not set up the unsafe data server : {err}")),
		}
	}

	pub fn get_addr(&self) -> String {
		let host = self
			.settings
			.domain
			.clone()
			.unwrap_or(String::from("127.0.0.1"));
		let port = self.program_state.lock().unwrap().http_port;
		format!("{host}:{port}")
	}
}
