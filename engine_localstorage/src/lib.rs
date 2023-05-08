#![allow(clippy::needless_return)]

use std::{
	collections::BTreeMap,
	sync::{Arc, Mutex},
};

use pontus_onyx::{item::Item, item::Path, EngineResponse, Method};

#[derive(Default)]
pub struct LocalStorageEngine {
	prefix: String,
}

pub struct EngineSettings {
	pub prefix: String,
}

#[async_trait::async_trait]
impl pontus_onyx::Engine for LocalStorageEngine {
	type Settings = EngineSettings;

	fn new(settings: Self::Settings) -> Self {
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

		Self {
			prefix: settings.prefix,
		}
	}

	async fn perform(&mut self, request: &pontus_onyx::Request) -> EngineResponse {
		log::debug!("performing {request:?}");

		let path = request.path.clone();
		let target_path = format!("{}/{path}", self.prefix);

		if request.method == Method::Put {
			let new_etag = pontus_onyx::item::Etag::from(format!("{}", uuid::Uuid::new_v4()));
			let new_last_modified =
				pontus_onyx::item::LastModified::from(time::OffsetDateTime::now_utc());

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

			let response = {
				let storage = match StorageProxy::new() {
					Ok(storage) => storage,
					Err(err) => {
						return EngineResponse::InternalError(err);
					}
				};
				let exists = storage.get_item(&target_path).unwrap().is_none();

				match storage.set_item(&target_path, serde_json::to_string(&new_item).unwrap()) {
					Ok(()) => {
						if exists {
							EngineResponse::UpdateSuccess(new_etag, new_last_modified)
						} else {
							EngineResponse::CreateSuccess(new_etag, new_last_modified)
						}
					}
					Err(err) => EngineResponse::InternalError(format!("{err:?}")),
				}
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
			let response = {
				let storage = match StorageProxy::new() {
					Ok(storage) => storage,
					Err(err) => {
						return EngineResponse::InternalError(err);
					}
				};

				if storage.get_item(&target_path).unwrap().is_none() {
					return EngineResponse::NotFound;
				}

				match storage.remove_item(&target_path) {
					Ok(()) => EngineResponse::DeleteSuccess,
					Err(err) => EngineResponse::InternalError(format!("{err:?}")),
				}
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
				let storage = match StorageProxy::new() {
					Ok(storage) => storage,
					Err(err) => {
						return EngineResponse::InternalError(err);
					}
				};

				match storage.get_item(&target_path) {
					Ok(Some(item)) => {
						let item: Item = serde_json::from_str(&item).unwrap();

						EngineResponse::GetSuccessDocument(if request.method != Method::Head {
							item
						} else {
							item.clone_without_content()
						})
					}
					Ok(None) => EngineResponse::NotFound,
					Err(err) => EngineResponse::InternalError(format!("{err:?}")),
				}
			} else if request.path.is_folder() {
				let storage = match StorageProxy::new() {
					Ok(storage) => storage,
					Err(err) => {
						return EngineResponse::InternalError(err);
					}
				};

				let mut response = match storage.get_item(&target_path) {
					Ok(Some(folder)) => {
						let folder: Item = serde_json::from_str(&folder).unwrap();

						EngineResponse::GetSuccessFolder {
							folder,
							children: BTreeMap::new(),
						}
					}
					Ok(None) => EngineResponse::NotFound,
					Err(err) => EngineResponse::InternalError(format!("{err:?}")),
				};

				if let EngineResponse::GetSuccessFolder {
					folder: _,
					children,
				} = &mut response
				{
					if request.method != Method::Head {
						for i in 0..storage.length().unwrap_or(0) {
							if let Ok(Some(key)) = storage.key(i) {
								if let Ok(Some(value)) = storage.get_item(&key) {
									if pontus_onyx::item::Path::try_from(&key)
										.unwrap()
										.is_direct_child(
											&pontus_onyx::item::Path::try_from(&target_path)
												.unwrap(),
										) {
										if let Some(key) =
											key.strip_prefix(&format!("{}/", self.prefix))
										{
											children.insert(
												pontus_onyx::item::Path::try_from(key)
													.unwrap()
													.clone(),
												serde_json::from_str::<Item>(&value)
													.unwrap()
													.clone(),
											);
										}
									}
								}
							}
						}
					}
				}

				response
			} else {
				EngineResponse::InternalError(String::from("path is not a folder nor a document"))
			}
		}
	}

	fn new_for_tests() -> Self {
		let prefix = format!("{}", uuid::Uuid::new_v4());
		let prefix_storage = format!("{prefix}/");

		let storage = StorageProxy::new().unwrap();

		storage
			.set_item(
				format!("{prefix_storage}folder_a/document.txt"),
				serde_json::to_string(&Item::Document {
					etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
					last_modified: Some(time::OffsetDateTime::now_utc().into()),
					content: Some(b"My Document Content Here (folder a)".into()),
					content_type: Some("text/html".into()),
				})
				.unwrap(),
			)
			.unwrap();

		storage
			.set_item(
				format!("{prefix_storage}folder_b/document.txt"),
				serde_json::to_string(&Item::Document {
					etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
					last_modified: Some(time::OffsetDateTime::now_utc().into()),
					content: Some(b"My Document Content Here (folder b)".into()),
					content_type: Some("text/html".into()),
				})
				.unwrap(),
			)
			.unwrap();

		storage
			.set_item(
				format!("{prefix_storage}folder_b/other_document.txt"),
				serde_json::to_string(&Item::Document {
					etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
					last_modified: Some(time::OffsetDateTime::now_utc().into()),
					content: Some(b"My Other Document Content Here (folder b)".into()),
					content_type: Some("text/html".into()),
				})
				.unwrap(),
			)
			.unwrap();

		storage
			.set_item(
				format!("{prefix_storage}folder_a/"),
				serde_json::to_string(&Item::Folder {
					etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
					last_modified: Some(time::OffsetDateTime::now_utc().into()),
				})
				.unwrap(),
			)
			.unwrap();

		storage
			.set_item(
				format!("{prefix_storage}folder_b/"),
				serde_json::to_string(&Item::Folder {
					etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
					last_modified: Some(time::OffsetDateTime::now_utc().into()),
				})
				.unwrap(),
			)
			.unwrap();

		storage
			.set_item(
				format!("{prefix_storage}public/folder_c/document.txt"),
				serde_json::to_string(&Item::Document {
					etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
					last_modified: Some(time::OffsetDateTime::now_utc().into()),
					content: Some(b"My Document Content Here (folder c)".into()),
					content_type: Some("text/html".into()),
				})
				.unwrap(),
			)
			.unwrap();

		storage
			.set_item(
				format!("{prefix_storage}public/folder_c/"),
				serde_json::to_string(&Item::Folder {
					etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
					last_modified: Some(time::OffsetDateTime::now_utc().into()),
				})
				.unwrap(),
			)
			.unwrap();

		storage
			.set_item(
				format!("{prefix_storage}public/"),
				serde_json::to_string(&Item::Folder {
					etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
					last_modified: Some(time::OffsetDateTime::now_utc().into()),
				})
				.unwrap(),
			)
			.unwrap();

		storage
			.set_item(
				format!("{prefix_storage}document.txt"),
				serde_json::to_string(&Item::Document {
					etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
					last_modified: Some(time::OffsetDateTime::now_utc().into()),
					content: Some(b"My Document Content Here (root)".into()),
					content_type: Some("text/html".into()),
				})
				.unwrap(),
			)
			.unwrap();

		storage
			.set_item(
				prefix_storage,
				serde_json::to_string(&Item::Folder {
					etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
					last_modified: Some(time::OffsetDateTime::now_utc().into()),
				})
				.unwrap(),
			)
			.unwrap();

		Self { prefix }
	}

	fn root_for_tests(&self) -> BTreeMap<Path, Item> {
		let mut result = BTreeMap::new();

		let storage = StorageProxy::new().unwrap();

		for i in 0..storage.length().unwrap_or(0) {
			if let Ok(Some(key)) = storage.key(i) {
				if let Ok(Some(value)) = storage.get_item(&key) {
					if let Some(key) = key.strip_prefix(&format!("{}/", self.prefix)) {
						result.insert(
							pontus_onyx::item::Path::try_from(key).unwrap().clone(),
							serde_json::from_str::<Item>(&value).unwrap().clone(),
						);
					}
				}
			}
		}

		return result;
	}
}

static MOCK: once_cell::sync::Lazy<Arc<Mutex<BTreeMap<String, String>>>> =
	once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(BTreeMap::new())));

#[derive(Clone)]
enum StorageProxy {
	Native(web_sys::Storage),
	Simulated,
}
impl StorageProxy {
	fn new() -> Result<Self, String> {
		if cfg!(test) {
			Ok(Self::Simulated)
		} else {
			let window = web_sys::window();
			if window.is_none() {
				return Err(String::from("there is no Window object available"));
			}

			let storage = window.unwrap().local_storage();
			if let Err(err) = storage {
				return Err(format!("can not get Storage object : {err:?}"));
			}
			let storage = storage.unwrap();
			if storage.is_none() {
				return Err(String::from("there is no localStorage object available"));
			}

			Ok(Self::Native(storage.unwrap()))
		}
	}

	fn get_item(&self, key: impl Into<String>) -> Result<Option<String>, wasm_bindgen::JsValue> {
		match self {
			StorageProxy::Native(inner) => inner.get_item(&key.into()),
			StorageProxy::Simulated => Ok(MOCK.lock().unwrap().get(&key.into()).map(Clone::clone)),
		}
	}

	fn set_item(
		&self,
		key: impl Into<String>,
		value: impl Into<String>,
	) -> Result<(), wasm_bindgen::JsValue> {
		match self {
			StorageProxy::Native(inner) => inner.set_item(&key.into(), &value.into()),
			StorageProxy::Simulated => {
				MOCK.lock().unwrap().insert(key.into(), value.into());

				Ok(())
			}
		}
	}

	fn remove_item(&self, key: impl Into<String>) -> Result<(), wasm_bindgen::JsValue> {
		match self {
			StorageProxy::Native(inner) => inner.remove_item(&key.into()),
			StorageProxy::Simulated => {
				MOCK.lock().unwrap().remove(&key.into());

				Ok(())
			}
		}
	}

	fn length(&self) -> Result<u32, wasm_bindgen::JsValue> {
		match self {
			StorageProxy::Native(inner) => inner.length(),
			StorageProxy::Simulated => Ok(MOCK.lock().unwrap().len() as u32),
		}
	}

	fn key(&self, index: u32) -> Result<Option<String>, wasm_bindgen::JsValue> {
		match self {
			StorageProxy::Native(inner) => inner.key(index),
			StorageProxy::Simulated => {
				Ok(MOCK.lock().unwrap().keys().nth(index.try_into().unwrap()))
					.map(|res| res.cloned())
			}
		}
	}
}

impl std::fmt::Debug for StorageProxy {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::Native(_) => f.write_str("Native"),
			Self::Simulated => f.write_fmt(format_args!("Simulated({:?})", MOCK.lock().unwrap())),
		}
	}
}

#[cfg(test)]
mod generic_tests;
