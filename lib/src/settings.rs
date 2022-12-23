pub struct DatabaseSettings {
	pub token_lifetime_seconds: Option<i64>,
}

impl Default for DatabaseSettings {
	fn default() -> Self {
		Self {
			token_lifetime_seconds: Some(30 * 60),
		}
	}
}
