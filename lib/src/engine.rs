use std::collections::BTreeMap;

use crate::item::Item;

#[async_trait::async_trait]
pub trait Engine {
	async fn perform(&mut self, request: &crate::Request) -> EngineResponse;

	fn new_for_tests() -> Self;
	fn root_for_tests(&self) -> BTreeMap<crate::item::Path, Item>;
}

#[derive(Debug, PartialEq)]
pub enum EngineResponse {
	GetSuccessDocument(Item),
	GetSuccessFolder {
		folder: Item,
		children: BTreeMap<crate::item::Path, Item>,
	},
	CreateSuccess(crate::item::Etag, crate::item::LastModified),
	UpdateSuccess(crate::item::Etag, crate::item::LastModified),
	DeleteSuccess,
	NotFound,
	InternalError(String),
}
impl EngineResponse {
	pub fn has_muted_database(&self) -> bool {
		match self {
			Self::GetSuccessDocument(_) => false,
			Self::GetSuccessFolder { .. } => false,
			Self::CreateSuccess(_, _) => true,
			Self::UpdateSuccess(_, _) => true,
			Self::DeleteSuccess => true,
			Self::NotFound => false,
			Self::InternalError(_) => false,
		}
	}
	pub fn get_new_etag(&self) -> Option<crate::item::Etag> {
		match self {
			Self::GetSuccessDocument(Item::Document { etag, .. }) => etag.clone(),
			Self::GetSuccessDocument(Item::Folder { etag, .. }) => etag.clone(),
			Self::GetSuccessFolder {
				folder: Item::Document { etag, .. },
				children: _,
			} => etag.clone(),
			Self::GetSuccessFolder {
				folder: Item::Folder { etag, .. },
				children: _,
			} => etag.clone(),
			Self::CreateSuccess(etag, _) => Some(etag.clone()),
			Self::UpdateSuccess(etag, _) => Some(etag.clone()),
			Self::DeleteSuccess => None,
			Self::NotFound => None,
			Self::InternalError(_) => None,
		}
	}
	pub fn get_last_modified(&self) -> Option<crate::item::LastModified> {
		match self {
			Self::GetSuccessDocument(Item::Document { last_modified, .. }) => last_modified.clone(),
			Self::GetSuccessDocument(Item::Folder { last_modified, .. }) => last_modified.clone(),
			Self::GetSuccessFolder {
				folder: Item::Document { last_modified, .. },
				children: _,
			} => last_modified.clone(),
			Self::GetSuccessFolder {
				folder: Item::Folder { last_modified, .. },
				children: _,
			} => last_modified.clone(),
			Self::CreateSuccess(_, last_modified) => Some(last_modified.clone()),
			Self::UpdateSuccess(_, last_modified) => Some(last_modified.clone()),
			Self::DeleteSuccess => None,
			Self::NotFound => None,
			Self::InternalError(_) => None,
		}
	}
}
