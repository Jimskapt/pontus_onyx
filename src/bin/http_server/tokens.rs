use rand::seq::IteratorRandom;
use rand::Rng;

use pontus_onyx::scope::Scope;

#[derive(Debug, Clone)]
pub struct AccessBearer {
	name: String,
	scopes: Vec<Scope>,
	client_id: String,
	username: String,
	emit_time: std::time::Instant,
}
impl AccessBearer {
	pub fn new(scopes: Vec<Scope>, client_id: &str, username: &str) -> Self {
		let mut name = String::new();

		let mut rng_limit = rand::thread_rng();
		for _ in 1..rng_limit.gen_range(128..256) {
			let mut rng_item = rand::thread_rng();
			name.push(
				crate::http_server::ACCESS_TOKEN_ALPHABET
					.chars()
					.choose(&mut rng_item)
					.unwrap(),
			);
		}
		name.push('=');

		Self {
			name,
			scopes,
			client_id: String::from(client_id),
			username: String::from(username),
			emit_time: std::time::Instant::now(),
		}
	}

	pub fn get_name(&self) -> &str {
		&self.name
	}
	pub fn get_scopes(&self) -> &[Scope] {
		&self.scopes
	}
	/*
	pub fn get_client_id(&self) -> &str {
		&self.client_id
	}
	*/
	pub fn get_username(&self) -> &str {
		&self.username
	}
	pub fn get_emit_time(&self) -> &std::time::Instant {
		&self.emit_time
	}
}

#[cfg(test)]
mod tests {
	use pontus_onyx::scope::{ScopeParsingError, ScopeRightType};

	#[test]
	fn c0ok0eil7m3() {
		assert_eq!(
			super::Scope::try_from("*:rw"),
			Ok(super::Scope {
				right_type: ScopeRightType::ReadWrite,
				module: String::from("*")
			})
		);
	}

	#[test]
	fn kn76nin3ppdf25t3p7zao() {
		assert_eq!(
			super::Scope::try_from("random:r"),
			Ok(super::Scope {
				right_type: ScopeRightType::Read,
				module: String::from("random")
			})
		);
	}

	#[test]
	fn sllj3xshcq266faixwpa() {
		assert_eq!(
			super::Scope::try_from("public:rw"),
			Err(ScopeParsingError::IncorrectModule(String::from("public")))
		);
	}

	#[test]
	fn mt1ns651q04kfc() {
		assert_eq!(
			super::Scope::try_from("wrong_char@:rw"),
			Err(ScopeParsingError::IncorrectModule(String::from(
				"wrong_char@"
			)))
		);
	}

	#[test]
	fn gy8bajrpald87() {
		assert_eq!(
			super::Scope::try_from("wrong:char:rw"),
			Err(ScopeParsingError::IncorrectFormat(String::from(
				"wrong:char:rw"
			)))
		);
	}

	#[test]
	fn jrx3s6biaha6ztxvollgn() {
		assert_eq!(
			super::Scope::try_from("random:wrong_right"),
			Err(ScopeParsingError::IncorrectRight(String::from(
				"wrong_right"
			)))
		);
	}
}
