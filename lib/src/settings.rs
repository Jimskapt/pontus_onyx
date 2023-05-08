#[derive(zeroize::Zeroize, zeroize::ZeroizeOnDrop)]
pub struct DatabaseSettings {
	#[zeroize(skip)]
	pub token_lifetime_seconds: Option<i64>,
	#[zeroize(skip)]
	pub userfile_path: Option<std::path::PathBuf>,
	pub encryption_key: Option<[u8; 32]>,
}

impl Default for DatabaseSettings {
	fn default() -> Self {
		Self {
			token_lifetime_seconds: Some(30 * 60),
			userfile_path: None,
			encryption_key: None,
		}
	}
}
