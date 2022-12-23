pub struct Response {
	pub request: crate::Request,
	pub status: ResponseStatus,
}

#[derive(Debug, PartialEq)]
pub enum ResponseStatus {
	Unallowed(crate::AccessError),
	Performed(crate::EngineResponse),
}
