use rand::Rng;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Settings {
	pub force_https: Option<bool>,
	pub domain: Option<String>,
	pub domain_suffix: Option<String>,
	pub port: usize,
	pub token_lifetime_seconds: Option<u64>,
	pub oauth_wait_seconds: Option<u64>,
	pub logfile_path: String,
	pub userfile_path: String,
	pub data_path: String,
	pub https: Option<SettingsHTTPS>,
}
impl Default for Settings {
	fn default() -> Self {
		Self {
			force_https: None,
			domain: Some(String::from("127.0.0.1")),
			domain_suffix: None,
			port: random_port_generation(),
			token_lifetime_seconds: Some(60 * 60),
			oauth_wait_seconds: Some(2),
			logfile_path: String::from("server.log"),
			userfile_path: String::from("users.bin"),
			data_path: String::from("data"),
			https: None,
		}
	}
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SettingsHTTPS {
	port: usize,
	keyfile_path: String,
	certfile_path: String,
	enable_hsts: bool,
}

fn random_port_generation() -> usize {
	let mut rng = rand::thread_rng();

	let port = rng.gen_range(1024..65535);

	port as usize
}
