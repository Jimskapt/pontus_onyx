pub struct Response {
	pub request: crate::Request,
	pub status: ResponseStatus,
}

#[derive(Debug, PartialEq)]
pub enum ResponseStatus {
	Unallowed(crate::AccessError),
	NoIfMatch(crate::Etag),
	IfNoneMatch(crate::Etag),
	Performed(crate::EngineResponse),
}
