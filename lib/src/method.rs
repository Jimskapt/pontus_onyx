#[derive(Debug, PartialEq, Clone)]
pub enum Method {
	Head,
	Get,
	Put,
	Delete,
}

#[cfg(feature = "actix_server")]
impl From<&actix_web::http::Method> for Method {
	fn from(input: &actix_web::http::Method) -> Self {
		match *input {
			actix_web::http::Method::HEAD => Self::Head,
			actix_web::http::Method::GET => Self::Get,
			actix_web::http::Method::POST => Self::Put,
			actix_web::http::Method::PUT => Self::Put,
			actix_web::http::Method::DELETE => Self::Delete,
			actix_web::http::Method::CONNECT => unimplemented!(),
			actix_web::http::Method::OPTIONS => unimplemented!(),
			actix_web::http::Method::PATCH => unimplemented!(),
			actix_web::http::Method::TRACE => unimplemented!(),
			_ => unimplemented!(),
		}
	}
}
