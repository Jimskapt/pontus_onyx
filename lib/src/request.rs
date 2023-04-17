#[cfg(feature = "actix_server")]
use actix_web::HttpMessage;

use crate::{item::Path, security::Origin, Limit, Method};

#[derive(Debug, PartialEq)]
pub struct Request {
	pub method: Method,
	pub path: Path,
	pub token: Option<crate::security::Token>,
	pub item: Option<crate::item::Item>,
	pub limits: Vec<Limit>,
	pub origin: Origin,
}

impl Request {
	pub fn new(
		new_method: impl Into<Method>,
		new_path: impl AsRef<Path>,
		origin: impl Into<Origin>,
	) -> Self {
		Self {
			method: new_method.into(),
			path: new_path.as_ref().clone(),
			token: None,
			item: None,
			limits: vec![],
			origin: origin.into(),
		}
	}
	pub fn head(new_path: impl AsRef<Path>, origin: impl Into<Origin>) -> Self {
		Self::new(Method::Head, new_path, origin)
	}
	pub fn get(new_path: impl AsRef<Path>, origin: impl Into<Origin>) -> Self {
		Self::new(Method::Get, new_path, origin)
	}
	pub fn put(new_path: impl AsRef<Path>, origin: impl Into<Origin>) -> Self {
		Self::new(Method::Put, new_path, origin)
	}
	pub fn delete(new_path: impl AsRef<Path>, origin: impl Into<Origin>) -> Self {
		Self::new(Method::Delete, new_path, origin)
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
	pub fn origin(mut self, new_origin: impl Into<Origin>) -> Self {
		self.origin = new_origin.into();

		return self;
	}
}

#[test]
fn full_constructor() {
	const PATH: &str = "/storage/user/test_app/my_data.json";
	const TOKEN: &str = "token";
	const IF_MATCH: &str = "good_etag";
	const IF_NONE_MATCH: &str = "bad_etag";
	const ORIGIN: &str = "test";
	const CONTENT: &[u8] = br#"{"test":"content"}"#;
	const CONTENT_TYPE: &str = "application/json";

	let request = Request::get(Path::try_from(PATH).unwrap(), ORIGIN)
		.token(TOKEN)
		.item(
			crate::item::Item::document()
				.content(CONTENT)
				.content_type(CONTENT_TYPE),
		)
		.add_limit(Limit::if_match(IF_MATCH))
		.add_limit(Limit::if_none_match(IF_NONE_MATCH));

	assert_eq!(
		request,
		Request {
			method: Method::Get,
			path: Path::try_from(PATH).unwrap(),
			token: Some(crate::security::Token::from(TOKEN)),
			item: Some(
				crate::item::Item::document()
					.content(CONTENT)
					.content_type(CONTENT_TYPE)
					.into()
			),
			limits: vec![
				Limit::IfMatch(crate::item::Etag::from(IF_MATCH)),
				Limit::IfNoneMatch(crate::item::Etag::from(IF_NONE_MATCH)),
			],
			origin: Origin::from(ORIGIN),
		}
	);
}

#[cfg(feature = "actix_server")]
pub async fn from_actix_request(
	actix_request: &actix_web::HttpRequest,
	actix_payload: &mut actix_web::dev::Payload,
) -> Result<self::Request, anyhow::Error> {
	let token = {
		if let Some(auth) = actix_request
			.headers()
			.get(actix_web::http::header::AUTHORIZATION)
		{
			if let Ok(auth) = auth.to_str() {
				auth.strip_prefix("Bearer ")
					.map(crate::security::Token::from)
			} else {
				None
			}
		} else {
			None
		}
	};

	let mut limits = vec![];
	match <actix_web::http::header::IfMatch as actix_web::http::header::Header>::parse(
		actix_request,
	) {
		Ok(actix_web::http::header::IfMatch::Any) => {
			limits.push(Limit::IfMatch(crate::item::Etag::from("*")));
		}
		Ok(actix_web::http::header::IfMatch::Items(items)) => {
			limits.append(
				&mut items
					.iter()
					.map(|entity| Limit::IfMatch(crate::item::Etag::from(entity.tag())))
					.collect(),
			);
		}
		Err(err) => {
			return Err(err.into());
		}
	}

	match <actix_web::http::header::IfNoneMatch as actix_web::http::header::Header>::parse(
		actix_request,
	) {
		Ok(actix_web::http::header::IfNoneMatch::Any) => {
			limits.push(Limit::IfNoneMatch(crate::item::Etag::from("*")));
		}
		Ok(actix_web::http::header::IfNoneMatch::Items(items)) => {
			limits.append(
				&mut items
					.iter()
					.map(|entity| Limit::IfNoneMatch(crate::item::Etag::from(entity.tag())))
					.collect(),
			);
		}
		Err(err) => {
			return Err(err.into());
		}
	}

	let origin =
		{
			if let Some(origin) = actix_request.headers().get(actix_web::http::header::ORIGIN) {
				if let Ok(origin) = origin.to_str() {
					Origin::from(origin)
				} else {
					return Err(anyhow::anyhow!(
						"can not convert origin header value in string"
					));
				}
			} else if cfg!(debug_assertions) {
				Origin::from("unknown (only allowed when running server in development mode for test purposes)")
			} else {
				return Err(anyhow::anyhow!("missing origin header"));
			}
		};

	let item = {
		if let Ok(content) = <actix_web::web::Bytes as actix_web::FromRequest>::from_request(
			actix_request,
			actix_payload,
		)
		.await
		{
			let content: &[u8] = &content;

			if content.is_empty() {
				None
			} else {
				Some(crate::item::Item::Document {
					etag: None,
					last_modified: None,
					content: Some(crate::item::Content::from(content.to_vec())),
					content_type: Some(crate::item::ContentType::from(
						actix_request.content_type(),
					)),
				})
			}
		} else {
			None
		}
	};

	match <actix_web::web::Path<String> as actix_web::FromRequest>::from_request(
		actix_request,
		actix_payload,
	)
	.await
	{
		Ok(path) => match crate::item::Path::try_from(path.into_inner()) {
			Ok(path) => Ok(self::Request {
				path,
				method: actix_request.method().into(),
				token,
				limits,
				item,
				origin,
			}),
			Err(err) => Err(err.into()),
		},
		Err(err) => Err(anyhow::anyhow!(err.to_string())),
	}
}

#[cfg(feature = "actix_server")]
#[tokio::test]
async fn convert_from_actix_request() {
	const REQUEST_PATH: &str = "/path/to/document";
	const TOKEN: &str = "dg4dv45dwx54wd6v84w6v4df";
	const IF_MATCH: &str = "AN_ETAG";
	const IF_NONE_MATCH: &str = "*";
	const CONTENT: &[u8] = b"Hello, world ?";
	const CONTENT_TYPE: &str = "text/plain";
	const ORIGIN: &str = "test";

	let actix_request = actix_web::test::TestRequest::put()
		.uri(&format!("/storage{REQUEST_PATH}"))
		.insert_header((
			actix_web::http::header::AUTHORIZATION,
			format!("Bearer {TOKEN}"),
		))
		.insert_header((actix_web::http::header::ORIGIN, ORIGIN))
		.param("path", REQUEST_PATH)
		.insert_header((actix_web::http::header::CONTENT_TYPE, CONTENT_TYPE))
		.insert_header((actix_web::http::header::IF_MATCH, format!("\"{IF_MATCH}\"")))
		.insert_header((actix_web::http::header::IF_NONE_MATCH, IF_NONE_MATCH))
		.insert_header((actix_web::http::header::CONTENT_LENGTH, CONTENT.len()))
		.insert_header((actix_web::http::header::ORIGIN, "test"))
		.to_http_request();

	let (mut sender, temp) = actix_http::h1::Payload::create(true);
	sender.feed_data(CONTENT.into());
	let mut payload = actix_web::dev::Payload::H1 { payload: temp };

	assert_eq!(
		from_actix_request(&actix_request, &mut payload)
			.await
			.unwrap(),
		Request {
			path: crate::item::Path::try_from(REQUEST_PATH).unwrap(),
			method: crate::Method::Put,
			token: Some(crate::security::Token::from(TOKEN)),
			limits: vec![
				crate::Limit::IfMatch(crate::item::Etag::from(IF_MATCH)),
				crate::Limit::IfNoneMatch(crate::item::Etag::from(IF_NONE_MATCH)),
			],
			item: Some(crate::item::Item::Document {
				etag: None,
				last_modified: None,
				content: Some(crate::item::Content::from(CONTENT)),
				content_type: Some(crate::item::ContentType::from(CONTENT_TYPE))
			}),
			origin: Origin::from(ORIGIN),
		}
	);
}
