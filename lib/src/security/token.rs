use crate::security::BearerAccess;

#[cfg(test)]
use crate::{item::Path, security::Origin};

#[derive(
	derivative::Derivative,
	PartialEq,
	Clone,
	PartialOrd,
	Ord,
	Eq,
	serde::Serialize,
	serde::Deserialize,
)]
#[derivative(Debug = "transparent")]
pub struct Token(pub String);

impl<T: Into<String>> From<T> for Token {
	fn from(value: T) -> Self {
		Self(value.into())
	}
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TokenMetadata {
	creation: time::OffsetDateTime,
	accesses: Vec<BearerAccess>,
}
impl TokenMetadata {
	pub fn new(accesses: Vec<BearerAccess>) -> Self {
		Self {
			creation: time::OffsetDateTime::now_utc(),
			accesses,
		}
	}
}
impl TokenMetadata {
	pub fn check(
		&self,
		token_lifetime_seconds: Option<i64>,
		request: &crate::Request,
	) -> Result<(), Vec<TokenValidityError>> {
		let mut errors = vec![];

		let lifetime = self.check_lifetime(token_lifetime_seconds);
		if let Err(error) = lifetime {
			errors.push(error);
		}

		let check_request = self.check_request(request);
		if let Err(error) = check_request {
			errors.append(
				&mut error
					.into_iter()
					.map(TokenValidityError::RequestError)
					.collect(),
			);
		}

		if errors.is_empty() {
			Ok(())
		} else {
			Err(errors)
		}
	}
	fn check_lifetime(
		&self,
		token_lifetime_seconds: Option<i64>,
	) -> Result<(), TokenValidityError> {
		if let Some(token_lifetime_seconds) = token_lifetime_seconds {
			let actual_lifetime = (self.creation - time::OffsetDateTime::now_utc())
				.whole_seconds()
				.abs();

			if actual_lifetime >= token_lifetime_seconds {
				return Err(TokenValidityError::LifetimeExpirated(actual_lifetime));
			}
		}

		Ok(())
	}
	fn check_request(&self, request: &crate::Request) -> Result<(), Vec<RequestValidityError>> {
		if self
			.accesses
			.iter()
			.any(|bearer| bearer.check_request(request).is_ok())
		{
			Ok(())
		} else {
			Err(self
				.accesses
				.iter()
				.map(|bearer| bearer.check_request(request).unwrap_err())
				.collect())
		}
	}
}

#[test]
fn expirated_token() {
	let token = TokenMetadata {
		creation: time::OffsetDateTime::now_utc() - time::Duration::seconds(60),
		accesses: vec![],
	};

	assert_eq!(token.check_lifetime(None), Ok(()));
	assert_eq!(token.check_lifetime(Some(5000)), Ok(()));
	assert_eq!(
		token.check_lifetime(Some(5)),
		Err(TokenValidityError::LifetimeExpirated(60))
	);
}

#[test]
fn multiple_modules() {
	let token = TokenMetadata {
		creation: time::OffsetDateTime::now_utc() - time::Duration::seconds(5000),
		accesses: vec![
			BearerAccess {
				right: crate::security::BearerAccessRight::ReadWrite,
				module: String::from("folder_a"),
			},
			BearerAccess {
				right: crate::security::BearerAccessRight::ReadWrite,
				module: String::from("folder_b"),
			},
		],
	};

	assert_eq!(
		token.check_request(&crate::Request::get(
			Path::try_from("folder_a/").unwrap(),
			Origin::from("test"),
		)),
		Ok(())
	);
	assert_eq!(
		token.check_request(&crate::Request::get(
			Path::try_from("folder_b/").unwrap(),
			Origin::from("test"),
		)),
		Ok(())
	);
	assert_eq!(
		token.check_request(&crate::Request::get(
			Path::try_from("folder_c/").unwrap(),
			Origin::from("test"),
		)),
		Err(vec![
			crate::security::RequestValidityError::OutOfModuleScope,
			crate::security::RequestValidityError::OutOfModuleScope,
		])
	);
}

#[test]
fn joker_modules() {
	let token = TokenMetadata {
		creation: time::OffsetDateTime::now_utc() - time::Duration::seconds(5000),
		accesses: vec![BearerAccess {
			right: crate::security::BearerAccessRight::ReadWrite,
			module: String::from("*"),
		}],
	};

	assert_eq!(
		token.check_request(&crate::Request::get(
			Path::try_from("folder_a/").unwrap(),
			Origin::from("test"),
		)),
		Ok(())
	);
	assert_eq!(
		token.check_request(&crate::Request::get(
			Path::try_from("folder_b/").unwrap(),
			Origin::from("test"),
		)),
		Ok(())
	);
	assert_eq!(
		token.check_request(&crate::Request::get(
			Path::try_from("folder_c/").unwrap(),
			Origin::from("test"),
		)),
		Ok(())
	);
	assert_eq!(
		token.check_request(&crate::Request::get(
			Path::try_from("folder_d/").unwrap(),
			Origin::from("test"),
		)),
		Ok(())
	);
	assert_eq!(
		token.check_request(&crate::Request::get(
			Path::try_from("folder_e/").unwrap(),
			Origin::from("test"),
		)),
		Ok(())
	);
	assert_eq!(
		token.check_request(&crate::Request::get(
			Path::try_from("folder_f/").unwrap(),
			Origin::from("test"),
		)),
		Ok(())
	);
}

#[derive(Debug, PartialEq)]
pub enum TokenValidityError {
	LifetimeExpirated(i64),
	RequestError(RequestValidityError),
}

#[derive(Debug, PartialEq)]
pub enum RequestValidityError {
	OutOfModuleScope,
	UnallowedMethod,
}
