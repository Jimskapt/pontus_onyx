pub trait Engine {
	fn perform(&mut self, request: &crate::Request) -> EngineResponse;

	fn new_for_tests() -> Self;
	fn root_for_tests(&self) -> crate::Item;
}

#[derive(Debug, PartialEq)]
pub enum EngineResponse {
	GetSuccess(crate::Item),
	CreateSuccess(crate::Etag, crate::LastModified),
	UpdateSuccess(crate::Etag, crate::LastModified),
	ContentNotChanged,
	DeleteSuccess(crate::Etag),
	NotFound,
	NoIfMatch(crate::Etag),
	IfNoneMatch(crate::Etag),
	InternalError(String),
}
impl EngineResponse {
	pub fn has_muted_database(&self) -> bool {
		match self {
			Self::GetSuccess(_) => false,
			Self::CreateSuccess(_, _) => true,
			Self::UpdateSuccess(_, _) => true,
			Self::ContentNotChanged => false,
			Self::DeleteSuccess(_) => true,
			Self::NotFound => false,
			Self::NoIfMatch(_) => false,
			Self::IfNoneMatch(_) => false,
			Self::InternalError(_) => false,
		}
	}
	pub fn get_new_etag(&self) -> Option<crate::Etag> {
		match self {
			Self::GetSuccess(crate::Item::Document { etag, .. }) => etag.clone(),
			Self::GetSuccess(crate::Item::Folder { etag, .. }) => etag.clone(),
			Self::CreateSuccess(etag, _) => Some(etag.clone()),
			Self::UpdateSuccess(etag, _) => Some(etag.clone()),
			Self::ContentNotChanged => None,
			Self::DeleteSuccess(_) => None,
			Self::NotFound => None,
			Self::NoIfMatch(_) => None,
			Self::IfNoneMatch(_) => None,
			Self::InternalError(_) => None,
		}
	}
	pub fn get_last_modified(&self) -> Option<crate::LastModified> {
		match self {
			Self::GetSuccess(crate::Item::Document { last_modified, .. }) => last_modified.clone(),
			Self::GetSuccess(crate::Item::Folder { last_modified, .. }) => last_modified.clone(),
			Self::CreateSuccess(_, last_modified) => Some(last_modified.clone()),
			Self::UpdateSuccess(_, last_modified) => Some(last_modified.clone()),
			Self::ContentNotChanged => None,
			Self::DeleteSuccess(_) => None,
			Self::NotFound => None,
			Self::NoIfMatch(_) => None,
			Self::IfNoneMatch(_) => None,
			Self::InternalError(_) => None,
		}
	}
}
