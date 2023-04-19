use std::sync::{Arc, Mutex};

pub fn run<T: pontus_onyx::Engine + Send + 'static>(
	settings: crate::settings::Settings,
	storage_db: Arc<Mutex<pontus_onyx::Database<T>>>,
	program_state: Arc<Mutex<crate::ProgramState>>,
	form_tokens: Arc<Mutex<Vec<crate::FormToken>>>,
) -> Result<std::thread::JoinHandle<Result<(), std::io::Error>>, String> {
	let host = settings.domain.clone().unwrap_or(String::from("127.0.0.1"));
	let port = program_state.lock().unwrap().http_port;
	let addr = format!("{host}:{port}");

	let bind = actix_web::HttpServer::new(move || {
		actix_web::App::new()
			.wrap(actix_web::middleware::Logger::default())
			.configure(crate::configure_server(
				settings.clone(),
				storage_db.clone(),
				program_state.clone(),
				form_tokens.clone(),
			))
	})
	.bind(addr.clone())
	.unwrap(); // TODO

	log::info!("starting unsafe server at http://{addr}");

	let run = bind.run();

	Ok(std::thread::spawn(move || {
		let sys = actix_web::rt::System::new();
		sys.block_on(run)
	}))
}
