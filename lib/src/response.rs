pub struct Response {
	pub request: crate::Request,
	pub status: ResponseStatus,
}

#[derive(Debug, PartialEq)]
pub enum ResponseStatus {
	Performed(crate::EngineResponse),
	Unallowed(crate::AccessError),
	NoIfMatch(crate::Etag),
	IfNoneMatch(crate::Etag),
	ContentNotChanged,
	NotSuitableForFolderItem,
	MissingItem,
	InternalError(String),
}
