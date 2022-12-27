use std::collections::BTreeMap;

use pontus_onyx::EngineResponse;

#[derive(Default)]
pub struct MemoryEngine {
	root: BTreeMap<pontus_onyx::ItemPath, pontus_onyx::Item>,
}

impl MemoryEngine {
	pub fn new() -> Self {
		let mut root = BTreeMap::new();
		root.insert(
			"".try_into().unwrap(),
			pontus_onyx::Item::Folder {
				etag: Some(pontus_onyx::Etag::from(format!("{}", uuid::Uuid::new_v4()))),
				last_modified: Some(pontus_onyx::LastModified::from(
					time::OffsetDateTime::now_utc(),
				)),
			},
		);

		Self { root }
	}
}

#[async_trait::async_trait]
impl pontus_onyx::Engine for MemoryEngine {
	async fn perform(&mut self, request: &pontus_onyx::Request) -> EngineResponse {
		if request.method == pontus_onyx::Method::Get || request.method == pontus_onyx::Method::Head
		{
			if request.path.is_document() {
				match self.root.get(&request.path) {
					Some(item) => EngineResponse::GetSuccessDocument(
						if request.method != pontus_onyx::Method::Head {
							item.clone()
						} else {
							item.clone_without_content()
						},
					),
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
					if request.method != pontus_onyx::Method::Head {
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
		} else {
			todo!()
		}
	}

	fn new_for_tests() -> Self {
		let mut root = BTreeMap::new();

		root.insert(
			"folder_a/document.txt".try_into().unwrap(),
			pontus_onyx::Item::Document {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
				content: Some(b"My Document Content Here (folder a)".into()),
				content_type: Some("text/html".into()),
			},
		);

		root.insert(
			"folder_b/document.txt".try_into().unwrap(),
			pontus_onyx::Item::Document {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
				content: Some(b"My Document Content Here (folder b)".into()),
				content_type: Some("text/html".into()),
			},
		);

		root.insert(
			"folder_b/other_document.txt".try_into().unwrap(),
			pontus_onyx::Item::Document {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
				content: Some(b"My Other Document Content Here (folder b)".into()),
				content_type: Some("text/html".into()),
			},
		);

		root.insert(
			"folder_a/".try_into().unwrap(),
			pontus_onyx::Item::Folder {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
			},
		);

		root.insert(
			"folder_b/".try_into().unwrap(),
			pontus_onyx::Item::Folder {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
			},
		);

		root.insert(
			"public/folder_c/document.txt".try_into().unwrap(),
			pontus_onyx::Item::Document {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
				content: Some(b"My Document Content Here (folder c)".into()),
				content_type: Some("text/html".into()),
			},
		);

		root.insert(
			"public/folder_c/".try_into().unwrap(),
			pontus_onyx::Item::Folder {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
			},
		);

		root.insert(
			"public/".try_into().unwrap(),
			pontus_onyx::Item::Folder {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
			},
		);

		root.insert(
			"document.txt".try_into().unwrap(),
			pontus_onyx::Item::Document {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
				content: Some(b"My Document Content Here (root)".into()),
				content_type: Some("text/html".into()),
			},
		);

		root.insert(
			"".try_into().unwrap(),
			pontus_onyx::Item::Folder {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
			},
		);

		Self { root }
	}

	fn root_for_tests(&self) -> BTreeMap<pontus_onyx::ItemPath, pontus_onyx::Item> {
		self.root.clone()
	}
}

#[cfg(test)]
mod generic_tests;
