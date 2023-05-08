use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as AsyncMutex;

pub fn run<T: pontus_onyx::Engine + Send + 'static>(
	settings: crate::settings::Settings,
	storage_db: Arc<AsyncMutex<pontus_onyx::Database<T>>>,
	program_state: Arc<Mutex<crate::ProgramState>>,
	form_tokens: Arc<Mutex<Vec<crate::FormToken>>>,
) -> Result<std::thread::JoinHandle<Result<(), std::io::Error>>, String> {
	let host = settings.domain.clone().unwrap_or(String::from("127.0.0.1"));
	let port = program_state.lock().unwrap().https_port;

	match port {
		Some(port) => match &settings.https {
			Some(settings_https) => match std::fs::File::open(&settings_https.keyfile_path) {
				Ok(keyfile_content) => match std::fs::File::open(&settings_https.certfile_path) {
					Ok(cert_content) => {
						let key_file = &mut std::io::BufReader::new(keyfile_content);
						let cert_file = &mut std::io::BufReader::new(cert_content);
						match rustls_pemfile::certs(cert_file) {
							Ok(cert_chain) => match rustls_pemfile::pkcs8_private_keys(key_file) {
								Ok(keys) => match keys.get(0) {
									Some(key) => {
										let server_config = rustls::ServerConfig::builder()
											.with_safe_defaults()
											.with_no_client_auth()
											.with_single_cert(
												vec![rustls::Certificate(
													cert_chain.get(0).unwrap().clone(),
												)],
												rustls::PrivateKey(key.clone()),
											);

										match server_config {
											Ok(server_config) => {
												let addr = format!("{host}:{port}");

												let bind =
													actix_web::HttpServer::new(move || {
														actix_web::App::new()
															.wrap(actix_web::middleware::Logger::default())
															.configure(crate::configure_server(
																settings.clone(),
																storage_db.clone(),
																program_state.clone(),
																form_tokens.clone(),
															))
													})
													.bind_rustls(addr.clone(), server_config);

												match bind {
													Ok(bind) => {
														log::info!("starting securised server at https://{addr}");

														let run = bind.run();

														Ok(std::thread::spawn(move || {
															let sys = actix_web::rt::System::new();
															sys.block_on(run)
														}))
													},
													Err(err) => {
														Err(format!("can not set up the securised server : {err}"))
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
										settings_https.certfile_path
									)),
								},
								Err(e) => Err(format!("can not read PKCS8 private key : {e}")),
							},
							Err(e) => Err(format!("can not read SSL certificate : {e}")),
						}
					}
					Err(e) => Err(format!(
						"can not open cert file `{}` : {e}",
						settings_https.certfile_path
					)),
				},
				Err(e) => Err(format!(
					"can not open key file `{}` : {e}",
					settings_https.keyfile_path
				)),
			},
			None => todo!(),
		},
		None => todo!(),
	}
}
