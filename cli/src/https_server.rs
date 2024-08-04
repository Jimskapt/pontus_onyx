use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as AsyncMutex;

pub struct HTTPSServer<T: pontus_onyx::Engine + Send + 'static> {
	settings: crate::settings::Settings,
	storage_db: Arc<AsyncMutex<pontus_onyx::Database<T>>>,
	program_state: Arc<Mutex<crate::ProgramState>>,
	form_tokens: Arc<Mutex<Vec<crate::FormToken>>>,
}

impl<T: pontus_onyx::Engine + Send + 'static> HTTPSServer<T> {
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
		match std::fs::File::open(&self.settings.https.as_ref().unwrap().keyfile_path) {
			Ok(keyfile_content) => {
				match std::fs::File::open(&self.settings.https.as_ref().unwrap().certfile_path) {
					Ok(cert_content) => {
						let key_file = &mut std::io::BufReader::new(keyfile_content);
						let cert_file = &mut std::io::BufReader::new(cert_content);
						match rustls_pemfile::certs(cert_file) {
							Ok(cert_chain) => match rustls_pemfile::pkcs8_private_keys(key_file) {
								Ok(keys) => match keys.first() {
									Some(key) => {
										let server_config = rustls::ServerConfig::builder()
											.with_safe_defaults()
											.with_no_client_auth()
											.with_single_cert(
												vec![rustls::Certificate(
													cert_chain.first().unwrap().clone(),
												)],
												rustls::PrivateKey(key.clone()),
											);

										match server_config {
											Ok(server_config) => {
												let addr = self.get_addr();

												let bind =
													actix_web::HttpServer::new(move || {
														actix_web::App::new()
																.wrap(actix_web::middleware::Logger::default())
																.configure(crate::configure_server(
																	self.settings.clone(),
																	self.storage_db.clone(),
																	self.program_state.clone(),
																	self.form_tokens.clone(),
																))
													})
													.bind_rustls(addr.clone(), server_config);

												match bind {
														Ok(bind) => {
															log::info!("starting securised data server at https://{addr}");

															let run = bind.run();

															Ok(std::thread::spawn(move || {
																let sys = actix_web::rt::System::new();
																sys.block_on(run)
															}))
														},
														Err(err) => {
															Err(format!("can not set up the securised data server : {err}"))
														}
													}
											}
											Err(e) => Err(format!(
												"can not insert certificate in server : {e}"
											)),
										}
									}
									None => Err(format!(
										"no private key found in {}",
										&self.settings.https.unwrap().certfile_path
									)),
								},
								Err(e) => Err(format!("can not read PKCS8 private key : {e}")),
							},
							Err(e) => Err(format!("can not read SSL certificate : {e}")),
						}
					}
					Err(e) => Err(format!(
						"can not open cert file `{}` : {e}",
						&self.settings.https.unwrap().certfile_path
					)),
				}
			}
			Err(e) => Err(format!(
				"can not open key file `{}` : {e}",
				&self.settings.https.unwrap().keyfile_path
			)),
		}
	}

	pub fn get_addr(&self) -> String {
		let host = self
			.settings
			.domain
			.clone()
			.unwrap_or(String::from("127.0.0.1"));
		let port = self.program_state.lock().unwrap().https_port.unwrap();

		format!("{host}:{port}")
	}
}
