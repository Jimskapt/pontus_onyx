use std::collections::BTreeMap;

#[derive(zeroize::Zeroize, zeroize::ZeroizeOnDrop)]
pub struct User {
	#[zeroize(skip)]
	pub username: String,
	pub password: String,
	#[zeroize(skip)]
	pub tokens: BTreeMap<crate::Token, TokenMetadata>,
}

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
				right: BearerAccessRight::ReadWrite,
				module: String::from("folder_a"),
			},
			BearerAccess {
				right: BearerAccessRight::ReadWrite,
				module: String::from("folder_b"),
			},
		],
	};

	assert_eq!(
		token.check_request(&crate::Request::get(crate::ItemPath::try_from("folder_a/").unwrap())),
		Ok(())
	);
	assert_eq!(
		token.check_request(&crate::Request::get(crate::ItemPath::try_from("folder_b/").unwrap())),
		Ok(())
	);
	assert_eq!(
		token.check_request(&crate::Request::get(crate::ItemPath::try_from("folder_c/").unwrap())),
		Err(vec![
			crate::RequestValidityError::OutOfModuleScope,
			crate::RequestValidityError::OutOfModuleScope,
		])
	);
}

pub struct BearerAccess {
	right: BearerAccessRight,
	module: String,
}

impl TryFrom<&str> for BearerAccess {
	type Error = BearerAccessConvertError;

	fn try_from(input: &str) -> Result<Self, Self::Error> {
		let mut temp = input.split(':');

		let module = temp.next();
		let right = temp.next();
		let remaining = temp.next();

		match remaining {
			None => match module {
				Some(module) => match right {
					Some(right) => {
						if module == "public" {
							return Err(BearerAccessConvertError::IncorrectModule(String::from(
								module,
							)));
						}

						let regex = regex::Regex::new("^[a-z0-9_]+$").unwrap();
						if module == "*" || regex.is_match(module) {
							let right = BearerAccessRight::try_from(right)?;
							let module = String::from(module);

							Ok(Self { right, module })
						} else {
							Err(BearerAccessConvertError::IncorrectModule(String::from(
								module,
							)))
						}
					}
					None => Err(BearerAccessConvertError::IncorrectFormat(String::from(
						input,
					))),
				},
				None => Err(BearerAccessConvertError::IncorrectFormat(String::from(
					input,
				))),
			},
			Some(_) => {
				return Err(BearerAccessConvertError::IncorrectFormat(String::from(
					input,
				)));
			}
		}
	}
}

impl BearerAccess {
	pub fn check_request(&self, request: &crate::Request) -> Result<(), RequestValidityError> {
		self.right.method_check(&request.method)?;

		if request
			.path
			.starts_with(&(self.module.clone() + "/").try_into().unwrap())
		{
			return Ok(());
		} else {
			return Err(RequestValidityError::OutOfModuleScope);
		}
	}
}

#[derive(Debug, PartialEq)]
pub enum BearerAccessConvertError {
	IncorrectModule(String),
	IncorrectFormat(String),
	IncorrectRight(String),
}

#[test]
fn check_request() {
	let bearer = BearerAccess {
		right: BearerAccessRight::ReadWrite,
		module: String::from("folder_a"),
	};

	assert_eq!(
		bearer.check_request(&crate::Request::get(crate::ItemPath::try_from("folder_a/").unwrap())),
		Ok(())
	);

	assert_eq!(
		bearer.check_request(&crate::Request::get(crate::ItemPath::try_from("folder_a").unwrap())),
		Err(RequestValidityError::OutOfModuleScope)
	);

	assert_eq!(
		bearer.check_request(&crate::Request::get(crate::ItemPath::try_from("folder_b/").unwrap())),
		Err(RequestValidityError::OutOfModuleScope)
	);

	assert_eq!(
		bearer.check_request(&crate::Request::get(crate::ItemPath::try_from("folder_b").unwrap())),
		Err(RequestValidityError::OutOfModuleScope)
	);
}

enum BearerAccessRight {
	Read,
	ReadWrite,
}

impl BearerAccessRight {
	pub fn method_check(&self, method: &crate::Method) -> Result<(), RequestValidityError> {
		if match self {
			Self::Read => {
				vec![crate::Method::Head, crate::Method::Get]
			}
			Self::ReadWrite => {
				vec![
					crate::Method::Head,
					crate::Method::Get,
					crate::Method::Put,
					crate::Method::Delete,
				]
			}
		}
		.contains(method)
		{
			Ok(())
		} else {
			Err(RequestValidityError::UnallowedMethod)
		}
	}
}

impl std::convert::TryFrom<&str> for BearerAccessRight {
	type Error = BearerAccessConvertError;

	fn try_from(input: &str) -> Result<Self, Self::Error> {
		let input = input.trim();

		if input == "rw" {
			Ok(Self::ReadWrite)
		} else if input == "r" {
			Ok(Self::Read)
		} else {
			Err(BearerAccessConvertError::IncorrectRight(String::from(
				input,
			)))
		}
	}
}

#[test]
fn method_check_valid_read() {
	assert_eq!(
		BearerAccessRight::Read.method_check(&crate::Method::Head),
		Ok(())
	);
	assert_eq!(
		BearerAccessRight::Read.method_check(&crate::Method::Get),
		Ok(())
	);
	assert_eq!(
		BearerAccessRight::Read.method_check(&crate::Method::Put),
		Err(RequestValidityError::UnallowedMethod)
	);
	assert_eq!(
		BearerAccessRight::Read.method_check(&crate::Method::Delete),
		Err(RequestValidityError::UnallowedMethod)
	);
}

#[test]
fn method_check_valid_readwrite() {
	assert_eq!(
		BearerAccessRight::ReadWrite.method_check(&crate::Method::Head),
		Ok(())
	);
	assert_eq!(
		BearerAccessRight::ReadWrite.method_check(&crate::Method::Get),
		Ok(())
	);
	assert_eq!(
		BearerAccessRight::ReadWrite.method_check(&crate::Method::Put),
		Ok(())
	);
	assert_eq!(
		BearerAccessRight::ReadWrite.method_check(&crate::Method::Delete),
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
