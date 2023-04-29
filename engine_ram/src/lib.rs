use std::collections::BTreeMap;

use pontus_onyx::{item::Item, item::Path, EngineResponse, Method};

#[derive(Default)]
pub struct MemoryEngine {
	root: BTreeMap<Path, Item>,
}

#[async_trait::async_trait]
impl pontus_onyx::Engine for MemoryEngine {
	type Settings = ();

	fn new(_: Self::Settings) -> Self {
		let mut root = BTreeMap::new();
		root.insert(
			pontus_onyx::item::ROOT_PATH.clone(),
			Item::Folder {
				etag: Some(pontus_onyx::item::Etag::from(format!(
					"{}",
					uuid::Uuid::new_v4()
				))),
				last_modified: Some(pontus_onyx::item::LastModified::from(
					time::OffsetDateTime::now_utc(),
				)),
			},
		);

		Self { root }
	}

	async fn perform(&mut self, request: &pontus_onyx::Request) -> EngineResponse {
		log::debug!("performing {request:?}");

		if request.method == Method::Put {
			let new_etag = pontus_onyx::item::Etag::from(format!("{}", uuid::Uuid::new_v4()));
			let new_last_modified =
				pontus_onyx::item::LastModified::from(time::OffsetDateTime::now_utc());
			let path = request.path.clone();

			let mut new_item = request.item.as_ref().unwrap().clone();
			match &mut new_item {
				Item::Document {
					etag,
					last_modified,
					content: _,
					content_type: _,
				} => {
					*etag = Some(new_etag.clone());
					*last_modified = Some(new_last_modified.clone());
				}
				Item::Folder {
					etag,
					last_modified,
				} => {
					*etag = Some(new_etag.clone());
					*last_modified = Some(new_last_modified.clone());
				}
			};

			let response = match self.root.insert(path.clone(), new_item) {
				Some(_) => EngineResponse::UpdateSuccess(new_etag, new_last_modified),
				None => EngineResponse::CreateSuccess(new_etag, new_last_modified),
			};

			if let Some(parent) = path.parent() {
				self.perform(
					&pontus_onyx::Request::put(
						parent,
						pontus_onyx::security::Origin::from("engine_ram_internals"),
					)
					.item(Item::Folder {
						etag: None,
						last_modified: None,
					}),
				)
				.await;
			}

			response
		} else if request.method == Method::Delete {
			let response = match self.root.remove(&request.path) {
				Some(_) => EngineResponse::DeleteSuccess,
				None => EngineResponse::NotFound,
			};

			if let Some(parent) = request.path.parent() {
				if let EngineResponse::GetSuccessFolder {
					folder: _,
					children,
				} = self
					.perform(&pontus_onyx::Request::get(
						&parent,
						pontus_onyx::security::Origin::from("engine_ram_internals"),
					))
					.await
				{
					if children.is_empty() {
						self.perform(&pontus_onyx::Request::delete(
							parent,
							pontus_onyx::security::Origin::from("engine_ram_internals"),
						))
						.await;
					}
				}
			}

			response
		} else {
			// GET & HEAD & others
			if request.path.is_document() {
				match self.root.get(&request.path) {
					Some(item) => {
						EngineResponse::GetSuccessDocument(if request.method != Method::Head {
							item.clone()
						} else {
							item.clone_without_content()
						})
					}
					None => EngineResponse::NotFound,
				}
			} else if request.path.is_folder() {
				let mut response = match self.root.get(&request.path) {
					Some(folder) => EngineResponse::GetSuccessFolder {
						folder: folder.clone(),
						children: BTreeMap::new(),
					},
					None => EngineResponse::NotFound,
				};

				if let EngineResponse::GetSuccessFolder {
					folder: _,
					children,
				} = &mut response
				{
					if request.method != Method::Head {
						*children = self
							.root
							.iter()
							.filter(|(key, _value)| key.is_direct_child(&request.path))
							.map(|(key, value)| (key.clone(), value.clone()))
							.collect();
					}
				}

				response
			} else {
				EngineResponse::InternalError(String::from("path is not a folder nor a document"))
			}
		}
	}

	fn new_for_tests() -> Self {
		let mut root = BTreeMap::new();

		root.insert(
			"folder_a/document.txt".try_into().unwrap(),
			Item::Document {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
				content: Some(b"My Document Content Here (folder a)".into()),
				content_type: Some("text/html".into()),
			},
		);

		root.insert(
			"folder_b/document.txt".try_into().unwrap(),
			Item::Document {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
				content: Some(b"My Document Content Here (folder b)".into()),
				content_type: Some("text/html".into()),
			},
		);

		root.insert(
			"folder_b/other_document.txt".try_into().unwrap(),
			Item::Document {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
				content: Some(b"My Other Document Content Here (folder b)".into()),
				content_type: Some("text/html".into()),
			},
		);

		root.insert(
			"folder_a/".try_into().unwrap(),
			Item::Folder {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
			},
		);

		root.insert(
			"folder_b/".try_into().unwrap(),
			Item::Folder {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
			},
		);

		root.insert(
			"public/folder_c/document.txt".try_into().unwrap(),
			Item::Document {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
				content: Some(b"My Document Content Here (folder c)".into()),
				content_type: Some("text/html".into()),
			},
		);

		root.insert(
			"public/folder_c/".try_into().unwrap(),
			Item::Folder {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
			},
		);

		root.insert(
			"public/".try_into().unwrap(),
			Item::Folder {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
			},
		);

		root.insert(
			"document.txt".try_into().unwrap(),
			Item::Document {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
				content: Some(b"My Document Content Here (root)".into()),
				content_type: Some("text/html".into()),
			},
		);

		root.insert(
			pontus_onyx::item::ROOT_PATH.clone(),
			Item::Folder {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
			},
		);

		Self { root }
	}

	fn root_for_tests(&self) -> BTreeMap<Path, Item> {
		self.root.clone()
	}
}

#[cfg(test)]
mod generic_tests;
