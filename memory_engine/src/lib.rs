use std::collections::BTreeMap;

#[derive(Default)]
pub struct MemoryEngine {
	root: BTreeMap<pontus_onyx::ItemPath, pontus_onyx::Item>,
}

impl MemoryEngine {
	pub fn new() -> Self {
		Self {
			root: BTreeMap::new(),
		}
	}
}

impl pontus_onyx::Engine for MemoryEngine {
	fn perform(&mut self, request: &pontus_onyx::Request) -> pontus_onyx::EngineResponse {
		pontus_onyx::EngineResponse::InternalError(format!("TODO"))
	}

	fn new_for_tests() -> Self {
		let mut root = BTreeMap::new();

		root.insert(
			"folder_a/document.txt".try_into().unwrap(),
			pontus_onyx::Item::Document {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
				content: Some(b"My Document Content Here.".into()),
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

		Self { root }
	}
	fn root_for_tests(&self) -> pontus_onyx::Item {
		todo!()
	}
}

#[cfg(test)]
mod generic_tests;
