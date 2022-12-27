#[derive(Debug, PartialEq)]
pub struct Request {
	pub method: crate::Method,
	pub path: crate::ItemPath,
	pub token: Option<crate::Token>,
	pub item: Option<crate::Item>,
	pub limits: Vec<crate::Limit>,
}

impl Request {
	pub fn new(
		new_method: impl Into<crate::Method>,
		new_path: impl AsRef<crate::ItemPath>,
	) -> Self {
		Self {
			method: new_method.into(),
			path: new_path.as_ref().clone(),
			token: None,
			item: None,
			limits: vec![],
		}
	}
	pub fn head(new_path: impl AsRef<crate::ItemPath>) -> Self {
		Self::new(crate::Method::Head, new_path)
	}
	pub fn get(new_path: impl AsRef<crate::ItemPath>) -> Self {
		Self::new(crate::Method::Get, new_path)
	}
	pub fn put(new_path: impl AsRef<crate::ItemPath>) -> Self {
		Self::new(crate::Method::Put, new_path)
	}
	pub fn delete(new_path: impl AsRef<crate::ItemPath>) -> Self {
		Self::new(crate::Method::Delete, new_path)
	}
	pub fn token(mut self, token: impl Into<crate::Token>) -> Self {
		self.token = Some(token.into());

		return self;
	}
	pub fn item(mut self, new_item: impl Into<crate::Item>) -> Self {
		self.item = Some(new_item.into());

		return self;
	}
	pub fn add_limit(mut self, new_limit: impl Into<crate::Limit>) -> Self {
		self.limits.push(new_limit.into());

		return self;
	}
}

#[test]
fn full_constructor() {
	let request =
		Request::get(crate::ItemPath::try_from("/storage/user/test_app/my_data.json").unwrap())
			.token("token")
			.item(
				crate::Item::document()
					.content(br#"{"test":"content"}"#)
					.content_type("application/json"),
			)
			.add_limit(crate::Limit::if_match("good_etag"))
			.add_limit(crate::Limit::if_none_match("bad_etag"));

	assert_eq!(
		request,
		Request {
			method: crate::Method::Get,
			path: crate::ItemPath::try_from("/storage/user/test_app/my_data.json").unwrap(),
			token: Some(crate::Token::from("token")),
			item: Some(
				crate::Item::document()
					.content(br#"{"test":"content"}"#)
					.content_type("application/json")
					.into()
			),
			limits: vec![
				crate::Limit::IfMatch(crate::Etag::from("good_etag")),
				crate::Limit::IfNoneMatch(crate::Etag::from("bad_etag")),
			]
		}
	);
}
