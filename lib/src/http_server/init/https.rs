use std::sync::{Arc, Mutex};

pub fn setup_and_run_https_server(
	settings: Arc<Mutex<super::Settings>>,
	database: Arc<Mutex<crate::database::Database>>,
	access_tokens: Arc<Mutex<Vec<crate::http_server::AccessBearer>>>,
	oauth_form_tokens: Arc<Mutex<Vec<crate::http_server::middlewares::OauthFormToken>>>,
	users: Arc<Mutex<crate::http_server::Users>>,
	program_state: Arc<Mutex<crate::http_server::ProgramState>>,
	logger: Arc<Mutex<charlie_buffalo::Logger>>,
	workspace_path: &std::path::Path,
	history_sender: Option<std::sync::mpsc::Sender<crate::http_server::DbEvent>>,
) {
	let settings_for_setup = settings.lock().unwrap().clone();

	match settings_for_setup.https {
		Some(settings_https) => {
			match std::fs::File::open(&settings_https.keyfile_path) {
				Ok(keyfile_content) => match std::fs::File::open(&settings_https.certfile_path) {
					Ok(cert_content) => {
						let key_file = &mut std::io::BufReader::new(keyfile_content);
						let cert_file = &mut std::io::BufReader::new(cert_content);
						match rustls_pemfile::certs(cert_file) {
							Ok(cert_chain) => match rustls_pemfile::pkcs8_private_keys(key_file) {
								Ok(keys) => {
									match keys.get(0) {
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
													let enable_hsts = settings_https.enable_hsts;
													let https_port = settings_https.port;

													let localhost = String::from("localhost");
													let domain = settings
														.lock()
														.unwrap()
														.domain
														.as_ref()
														.unwrap_or_else(|| &localhost)
														.clone();

													let program_state_for_server =
														program_state.clone();
													let workspace_path_for_server =
														workspace_path.to_path_buf();
													let logger_for_server = logger.clone();
													match actix_web::HttpServer::new(move || {
														actix_web::App::new()
															.wrap(crate::http_server::middlewares::Hsts {
																enable: enable_hsts,
															})
															.wrap(crate::http_server::middlewares::Auth {
																logger: logger_for_server.clone(),
															})
															.wrap(crate::http_server::middlewares::Logger {
																logger: logger_for_server.clone(),
															})
															.configure(
																crate::http_server::configure_server(
																	settings.clone(),
																	database.clone(),
																	access_tokens.clone(),
																	oauth_form_tokens.clone(),
																	users.clone(),
																	program_state_for_server.clone(),
																	logger_for_server.clone(),
																	&workspace_path_for_server,
																	match &history_sender {
																		Some(history_sender) => Some(history_sender.clone()),
																		None => None,
																	}
																)
															)
													})
													.bind_rustls(
														format!("{domain}:{https_port}"),
														server_config,
													) {
														Ok(serverd) => {
															logger.lock().unwrap().push(
																vec![
																	(String::from("event"), String::from("setup")),
																	(String::from("module"), String::from("https")),
																	(String::from("level"), String::from("INFO")),
																],
																Some(&format!("API should now listen to https://{domain}:{https_port}/")),
															);

															program_state
																.lock()
																.unwrap()
																.https_mode = true;

															let https_server = serverd.run();

															std::thread::spawn(move || {
																let sys =
																	actix_web::rt::System::new();
																sys.block_on(https_server)
															});
														}
														Err(e) => {
															logger.lock().unwrap().push(
																vec![
																	(String::from("event"), String::from("setup")),
																	(String::from("module"), String::from("https")),
																	(String::from("level"), String::from("ERROR")),
																],
																Some(&format!("can not set up HTTPS server : {}",
																e)),
															);
														}
													}
												}
												Err(e) => {
													logger.lock().unwrap().push(
														vec![
															(String::from("event"), String::from("setup")),
															(String::from("module"), String::from("https")),
															(String::from("level"), String::from("ERROR")),
														],
														Some(&format!("can not insert certificate in server : {}",
														e)),
													);
												}
											}
										}
										None => {
											logger.lock().unwrap().push(
												vec![
													(String::from("event"), String::from("setup")),
													(String::from("module"), String::from("https")),
													(String::from("level"), String::from("ERROR")),
												],
												Some("no private key found"),
											);
										}
									}
								}
								Err(e) => {
									logger.lock().unwrap().push(
										vec![
											(String::from("event"), String::from("setup")),
											(String::from("module"), String::from("https")),
											(String::from("level"), String::from("ERROR")),
										],
										Some(&format!("can not read PKCS8 private key : {}", e)),
									);
								}
							},
							Err(e) => {
								logger.lock().unwrap().push(
									vec![
										(String::from("event"), String::from("setup")),
										(String::from("module"), String::from("https")),
										(String::from("level"), String::from("ERROR")),
									],
									Some(&format!("can not read SSL certificate : {}", e)),
								);
							}
						}
					}
					Err(e) => {
						logger.lock().unwrap().push(
							vec![
								(String::from("event"), String::from("setup")),
								(String::from("module"), String::from("https")),
								(String::from("level"), String::from("ERROR")),
							],
							Some(&format!(
								"can not open cert file `{}` : {}",
								settings_https.certfile_path, e
							)),
						);
					}
				},
				Err(e) => {
					logger.lock().unwrap().push(
						vec![
							(String::from("event"), String::from("setup")),
							(String::from("module"), String::from("https")),
							(String::from("level"), String::from("ERROR")),
						],
						Some(&format!(
							"can not open key file `{}` : {}",
							settings_https.keyfile_path, e
						)),
					);
				}
			}
		}
		None => {
			logger.lock().unwrap().push(
				vec![
					(String::from("event"), String::from("setup")),
					(String::from("module"), String::from("https")),
					(String::from("level"), String::from("ERROR")),
				],
				Some("no HTTPS settings found"),
			);
		}
	}
}
