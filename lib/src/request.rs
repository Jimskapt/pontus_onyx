use crate::{item::Path, Limit, Method};

#[derive(Debug, PartialEq)]
pub struct Request {
	pub method: Method,
	pub path: Path,
	pub token: Option<crate::security::Token>,
	pub item: Option<crate::item::Item>,
	pub limits: Vec<Limit>,
}

impl Request {
	pub fn new(new_method: impl Into<Method>, new_path: impl AsRef<Path>) -> Self {
		Self {
			method: new_method.into(),
			path: new_path.as_ref().clone(),
			token: None,
			item: None,
			limits: vec![],
		}
	}
	pub fn head(new_path: impl AsRef<Path>) -> Self {
		Self::new(Method::Head, new_path)
	}
	pub fn get(new_path: impl AsRef<Path>) -> Self {
		Self::new(Method::Get, new_path)
	}
	pub fn put(new_path: impl AsRef<Path>) -> Self {
		Self::new(Method::Put, new_path)
	}
	pub fn delete(new_path: impl AsRef<Path>) -> Self {
		Self::new(Method::Delete, new_path)
	}
	pub fn token(mut self, token: impl Into<crate::security::Token>) -> Self {
		self.token = Some(token.into());

		return self;
	}
	pub fn item(mut self, new_item: impl Into<crate::item::Item>) -> Self {
		self.item = Some(new_item.into());

		return self;
	}
	pub fn add_limit(mut self, new_limit: impl Into<Limit>) -> Self {
		self.limits.push(new_limit.into());

		return self;
	}
}

#[test]
fn full_constructor() {
	let request = Request::get(Path::try_from("/storage/user/test_app/my_data.json").unwrap())
		.token("token")
		.item(
			crate::item::Item::document()
				.content(br#"{"test":"content"}"#)
				.content_type("application/json"),
		)
		.add_limit(Limit::if_match("good_etag"))
		.add_limit(Limit::if_none_match("bad_etag"));

	assert_eq!(
		request,
		Request {
			method: Method::Get,
			path: Path::try_from("/storage/user/test_app/my_data.json").unwrap(),
			token: Some(crate::security::Token::from("token")),
			item: Some(
				crate::item::Item::document()
					.content(br#"{"test":"content"}"#)
					.content_type("application/json")
					.into()
			),
			limits: vec![
				Limit::IfMatch(crate::item::Etag::from("good_etag")),
				Limit::IfNoneMatch(crate::item::Etag::from("bad_etag")),
			]
		}
	);
}
