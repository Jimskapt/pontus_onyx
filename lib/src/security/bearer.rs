use crate::{security::RequestValidityError, Method, Request};

pub struct BearerAccess {
	pub right: BearerAccessRight,
	pub module: String,
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
	pub fn check_request(&self, request: &Request) -> Result<(), RequestValidityError> {
		self.right.method_check(&request.method)?;

		if self.module == "*" {
			return Ok(());
		} else if request
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
		bearer.check_request(&Request::get(
			crate::item::Path::try_from("folder_a/").unwrap()
		)),
		Ok(())
	);

	assert_eq!(
		bearer.check_request(&Request::get(
			crate::item::Path::try_from("folder_a").unwrap()
		)),
		Err(RequestValidityError::OutOfModuleScope)
	);

	assert_eq!(
		bearer.check_request(&Request::get(
			crate::item::Path::try_from("folder_b/").unwrap()
		)),
		Err(RequestValidityError::OutOfModuleScope)
	);

	assert_eq!(
		bearer.check_request(&Request::get(
			crate::item::Path::try_from("folder_b").unwrap()
		)),
		Err(RequestValidityError::OutOfModuleScope)
	);
}

pub enum BearerAccessRight {
	Read,
	ReadWrite,
}

impl BearerAccessRight {
	pub fn method_check(&self, method: &Method) -> Result<(), RequestValidityError> {
		if match self {
			Self::Read => {
				vec![Method::Head, Method::Get]
			}
			Self::ReadWrite => {
				vec![Method::Head, Method::Get, Method::Put, Method::Delete]
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
	assert_eq!(BearerAccessRight::Read.method_check(&Method::Head), Ok(()));
	assert_eq!(BearerAccessRight::Read.method_check(&Method::Get), Ok(()));
	assert_eq!(
		BearerAccessRight::Read.method_check(&Method::Put),
		Err(RequestValidityError::UnallowedMethod)
	);
	assert_eq!(
		BearerAccessRight::Read.method_check(&Method::Delete),
		Err(RequestValidityError::UnallowedMethod)
	);
}

#[test]
fn method_check_valid_readwrite() {
	assert_eq!(
		BearerAccessRight::ReadWrite.method_check(&Method::Head),
		Ok(())
	);
	assert_eq!(
		BearerAccessRight::ReadWrite.method_check(&Method::Get),
		Ok(())
	);
	assert_eq!(
		BearerAccessRight::ReadWrite.method_check(&Method::Put),
		Ok(())
	);
	assert_eq!(
		BearerAccessRight::ReadWrite.method_check(&Method::Delete),
		Ok(())
	);
}
