use crate::ProgramState;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Settings {
	pub domain: Option<String>,
	pub domain_suffix: Option<String>,
	pub port: Option<usize>,
	pub admin_ui_port: Option<usize>,
	pub token_lifetime_seconds: Option<u64>,
	pub oauth_wait_seconds: Option<u64>,
	pub logfile_path: String,
	pub userfile_path: String,
	pub data_path: String,
	pub https: Option<SettingsHTTPS>,
	pub custom_encryption_key: Option<[u8; 32]>,
}
impl Default for Settings {
	fn default() -> Self {
		Self {
			domain: Some(String::from("127.0.0.1")),
			domain_suffix: None,
			port: None,
			admin_ui_port: None,
			token_lifetime_seconds: Some(60 * 60),
			oauth_wait_seconds: Some(2),
			logfile_path: String::from("server.log"),
			userfile_path: String::from("users.bin"),
			data_path: String::from("data"),
			https: None,
			custom_encryption_key: None,
		}
	}
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SettingsHTTPS {
	pub port: Option<usize>,
	pub keyfile_path: String,
	pub certfile_path: String,
	pub enable_hsts: bool,
}

pub fn build_server_address(settings: &Settings, program_state: &ProgramState) -> String {
	let localhost = String::from("localhost");

	let mut protocol = String::from("http");
	if program_state.https_port.is_some() {
		protocol += "s";
	}

	let mut domain = settings.domain.as_ref().unwrap_or(&localhost).clone();
	if let Some(force_domain) = &settings.domain {
		if !force_domain.trim().is_empty() {
			domain = force_domain.clone();
		}
	}

	let port = if let Some(https_port) = program_state.https_port {
		if https_port != 443 {
			format!(":{}", https_port)
		} else {
			String::new()
		}
	} else if program_state.http_port != 80 {
		format!(":{}", program_state.http_port)
	} else {
		String::new()
	};

	let mut domain_suffix = String::new();
	if let Some(suffix) = &settings.domain_suffix {
		if !suffix.trim().is_empty() && !suffix.trim().ends_with('/') {
			domain_suffix = format!("{}/", suffix.trim())
		} else {
			domain_suffix = String::from(suffix.trim())
		}
	}

	format!("{}://{}{}/{}", protocol, domain, port, domain_suffix)
}

#[test]
fn pbw1cgzctiqe163() {
	let settings = Settings::default();
	let state = ProgramState {
		http_port: 8743,
		https_port: None,
		admin_ui_port: 7654,
		settings_path: std::path::PathBuf::from(""),
	};

	assert_eq!(
		build_server_address(&settings, &state),
		format!(
			"{}://{}:{}/{}",
			"http",
			settings.domain.unwrap_or_else(|| String::from("localhost")),
			state.http_port,
			""
		)
	);
}

#[test]
fn ykf0gcnr7z2ko4wtx8uub() {
	let mut settings = Settings::default();
	settings.domain_suffix = Some(String::from("test"));
	let state = ProgramState {
		http_port: 8743,
		https_port: None,
		admin_ui_port: 7654,
		settings_path: std::path::PathBuf::from(""),
	};

	assert_eq!(
		build_server_address(&settings, &state),
		format!(
			"{}://{}:{}/{}",
			"http",
			settings.domain.unwrap_or_else(|| String::from("localhost")),
			state.http_port,
			"test/"
		)
	);
}

#[test]
fn wxpy6tncuwbbavvxi() {
	let mut settings = Settings::default();
	settings.domain_suffix = Some(String::from("test/"));
	let state = ProgramState {
		http_port: 8743,
		https_port: None,
		admin_ui_port: 7654,
		settings_path: std::path::PathBuf::from(""),
	};

	assert_eq!(
		build_server_address(&settings, &state),
		format!(
			"{}://{}:{}/{}",
			"http",
			settings.domain.unwrap_or_else(|| String::from("localhost")),
			state.http_port,
			"test/"
		)
	);
}

#[test]
fn fpfxwrixa1jz7t() {
	let mut settings = Settings::default();
	settings.https = Some(SettingsHTTPS {
		port: Some(2467),
		keyfile_path: String::from("watever"),
		certfile_path: String::from("watever"),
		enable_hsts: true,
	});
	let state = ProgramState {
		http_port: 8743,
		https_port: Some(8743),
		admin_ui_port: 7654,
		settings_path: std::path::PathBuf::from(""),
	};

	assert_eq!(
		build_server_address(&settings, &state),
		format!(
			"{}://{}:{}/{}",
			"https",
			settings.domain.unwrap_or_else(|| String::from("localhost")),
			state.https_port.unwrap(),
			""
		)
	);
}

#[test]
fn xtgfpc3x1zcmb() {
	let domain = String::from("example.com");
	let mut settings = Settings::default();
	settings.domain = Some(domain.clone());
	let state = ProgramState {
		http_port: 8743,
		https_port: None,
		admin_ui_port: 7654,
		settings_path: std::path::PathBuf::from(""),
	};

	assert_eq!(
		build_server_address(&settings, &state),
		format!(
			"{}://{}/{}",
			"http",
			format!("{}:{}", domain, state.http_port),
			""
		)
	);
}

#[test]
fn ekkvpuijzifxc() {
	let domain = String::from("example.com");
	let mut settings = Settings::default();
	settings.domain = Some(domain.clone());
	settings.https = Some(SettingsHTTPS {
		port: Some(2467),
		keyfile_path: String::from("watever"),
		certfile_path: String::from("watever"),
		enable_hsts: true,
	});
	let state = ProgramState {
		http_port: 80,
		https_port: Some(2467),
		admin_ui_port: 7654,
		settings_path: std::path::PathBuf::from(""),
	};

	assert_eq!(
		build_server_address(&settings, &state),
		format!(
			"{}://{}/{}",
			"https",
			format!("{}:{}", domain, settings.https.unwrap().port.unwrap()),
			""
		)
	);
}

#[test]
fn bj8n5zhu2oaaed55561ygk() {
	let domain = String::from("example.com");
	let mut settings = Settings::default();
	settings.domain = Some(domain.clone());
	settings.port = Some(80);
	settings.https = None;
	let state = ProgramState {
		http_port: 80,
		https_port: None,
		admin_ui_port: 7654,
		settings_path: std::path::PathBuf::from(""),
	};

	assert_eq!(
		build_server_address(&settings, &state),
		format!("{}://{}/{}", "http", domain, "")
	);
}

#[test]
fn d434yaaxfqcnd4j() {
	let domain = String::from("example.com");
	let mut settings = Settings::default();
	settings.domain = Some(domain.clone());
	settings.port = Some(80);
	settings.https = Some(SettingsHTTPS {
		keyfile_path: String::from("whatever"),
		certfile_path: String::from("whatever"),
		enable_hsts: true,
		port: Some(443),
	});
	let state = ProgramState {
		http_port: 80,
		https_port: Some(443),
		admin_ui_port: 7654,
		settings_path: std::path::PathBuf::from(""),
	};

	assert_eq!(
		build_server_address(&settings, &state),
		format!("{}://{}/{}", "https", domain, "")
	);
}
