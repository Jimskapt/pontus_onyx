#![allow(clippy::needless_return)]

use std::collections::BTreeMap;

use pontus_onyx::{item::Item, item::Path, EngineResponse, Method};

#[derive(Default)]
pub struct FileSystemEngine {
	root_path: std::path::PathBuf,
}

pub struct EngineSettings {
	pub path: std::path::PathBuf,
}

#[async_trait::async_trait]
impl pontus_onyx::Engine for FileSystemEngine {
	type Settings = EngineSettings;

	fn new(settings: Self::Settings) -> Self {
		std::fs::create_dir_all(&settings.path).unwrap();

		Self {
			root_path: settings.path,
		}
	}

	async fn perform(&mut self, request: &pontus_onyx::Request) -> EngineResponse {
		log::debug!("performing {request:?}");

		let file_exists = self
			.root_path
			.join(&format!("{}", request.path.clone()))
			.exists();

		if request.method == Method::Put {
			let new_etag = pontus_onyx::item::Etag::from(format!("{}", uuid::Uuid::new_v4()));
			let new_last_modified =
				pontus_onyx::item::LastModified::from(time::OffsetDateTime::now_utc());
			let path = request.path.clone();

			let mut new_content = pontus_onyx::item::Content::from(&[]);

			let mut new_item = request.item.as_ref().unwrap().clone();
			match &mut new_item {
				Item::Document {
					etag,
					last_modified,
					content,
					content_type: _,
				} => {
					*etag = Some(new_etag.clone());
					*last_modified = Some(new_last_modified.clone());
					if let Some(content) = content {
						new_content = content.clone();
					}
				}
				Item::Folder {
					etag,
					last_modified,
				} => {
					*etag = Some(new_etag.clone());
					*last_modified = Some(new_last_modified.clone());
				}
			};

			let response = if file_exists {
				EngineResponse::UpdateSuccess(new_etag, new_last_modified)
			} else {
				EngineResponse::CreateSuccess(new_etag, new_last_modified)
			};

			let new_item_toml = match toml::to_string_pretty(&new_item.clone_without_content()) {
				Ok(res) => res,
				Err(err) => {
					log::error!("{}", err);
					return EngineResponse::InternalError(String::from(
						"error while write data on disk",
					));
				}
			};

			std::fs::create_dir_all(self.root_path.join(format!("{}", path)).parent().unwrap())
				.unwrap();

			let response = match std::fs::write(
				&self.root_path.join(format!("{}", path)),
				new_content.into_inner(),
			)
			.and(std::fs::write(
				&self
					.root_path
					.join(format!("{}", path.as_datafile(".itemdata.toml"))),
				new_item_toml,
			)) {
				Ok(()) => response,
				Err(err) => {
					log::error!("{}", err);
					return EngineResponse::InternalError(String::from(
						"error while write data on disk",
					));
				}
			};

			let mut path = path.parent();
			while let Some(parent) = path {
				self.perform(
					&pontus_onyx::Request::put(
						&parent,
						pontus_onyx::security::Origin::from("engine_filesytem_internals"),
					)
					.item(Item::Folder {
						etag: None,
						last_modified: None,
					}),
				)
				.await;

				path = parent.parent();
			}

			response
		} else if request.method == Method::Delete {
			if !self.root_path.join(format!("{}", request.path)).exists() {
				return EngineResponse::NotFound;
			}

			let response = match std::fs::remove_file(
				self.root_path
					.join(format!("{}", request.path.as_datafile(".itemdata.toml"))),
			)
			.and({
				if request.path.is_folder() {
					std::fs::remove_dir(self.root_path.join(format!("{}", request.path)))
				} else {
					std::fs::remove_file(self.root_path.join(format!("{}", request.path)))
				}
			}) {
				Ok(()) => EngineResponse::DeleteSuccess,
				Err(err) => {
					log::error!("{}", err);
					return EngineResponse::InternalError(String::from(
						"error while write data on disk",
					));
				}
			};

			let mut path = request.path.parent();
			while let Some(parent) = path {
				if parent != pontus_onyx::item::ROOT_PATH {
					if let EngineResponse::GetSuccessFolder {
						folder: _,
						children,
					} = self
						.perform(&pontus_onyx::Request::get(
							&parent,
							pontus_onyx::security::Origin::from("engine_filesytem_internals"),
						))
						.await
					{
						if children.is_empty() {
							self.perform(&pontus_onyx::Request::delete(
								&parent,
								pontus_onyx::security::Origin::from("engine_filesytem_internals"),
							))
							.await;
						}
					}
				}

				path = parent.parent();
			}

			response
		} else {
			// GET & HEAD & others
			if request.path.is_document() {
				let content = if request.method == pontus_onyx::Method::Head {
					if self.root_path.join(format!("{}", request.path)).exists() {
						Ok(None)
					} else {
						Err(std::io::Error::new(std::io::ErrorKind::NotFound, ""))
					}
				} else {
					match std::fs::read(self.root_path.join(format!("{}", request.path))) {
						Ok(content) => Ok(Some(pontus_onyx::item::Content::from(content))),
						Err(err) => Err(err),
					}
				};

				match content {
					Ok(content) => {
						match std::fs::read_to_string(
							self.root_path
								.join(format!("{}", request.path.as_datafile(".itemdata.toml"))),
						)
						.map(|data| toml::from_str::<Item>(&data))
						{
							Ok(Ok(Item::Document {
								etag,
								last_modified,
								content: _,
								content_type,
							})) => EngineResponse::GetSuccessDocument(Item::Document {
								etag,
								last_modified,
								content,
								content_type,
							}),
							Ok(Ok(Item::Folder { .. })) => EngineResponse::InternalError(
								String::from("expected document itemdata, but get folder itemdata"),
							),
							Ok(Err(err)) => {
								log::error!("{}", err);
								return EngineResponse::InternalError(String::from(
									"error while convert itemdata",
								));
							}
							Err(err) => {
								log::error!("{}", &err);
								return EngineResponse::InternalError(String::from(
									"error while reading itemdata file",
								));
							}
						}
					}
					Err(err) => {
						if err.kind() == std::io::ErrorKind::NotFound {
							EngineResponse::NotFound
						} else {
							log::error!("{}", err);
							return EngineResponse::InternalError(String::from(
								"error while reading content file",
							));
						}
					}
				}
			} else if request.path.is_folder() {
				let folder_path = self.root_path.join(format!("{}", request.path));

				if !std::path::Path::new(&folder_path).exists() {
					EngineResponse::NotFound
				} else {
					let folder_data = toml::from_str(
						&std::fs::read_to_string(
							self.root_path
								.join(format!("{}", request.path.as_datafile(".itemdata.toml"))),
						)
						.unwrap(),
					)
					.unwrap();

					let mut children = BTreeMap::new();
					if request.method != Method::Head {
						for el in std::fs::read_dir(&folder_path).unwrap() {
							let entry = el.unwrap();

							if !entry.file_name().to_str().unwrap().contains(".itemdata.") {
								let item_path = pontus_onyx::item::Path::try_from(
									format!(
										"{}{}",
										entry.path().display(),
										if entry.metadata().unwrap().is_dir() {
											"/"
										} else {
											""
										}
									)
									.replace('\\', "/"),
								)
								.unwrap();

								let itemdata: Item = toml::from_str(
									&std::fs::read_to_string(format!(
										"{}",
										item_path.as_datafile(".itemdata.toml")
									))
									.unwrap(),
								)
								.unwrap();

								let itemdata = if item_path.is_document() {
									itemdata
										.content(std::fs::read(format!("{}", item_path)).unwrap())
								} else {
									itemdata
								};

								children.insert(
									pontus_onyx::item::Path::try_from(format!(
										"{}{}",
										request.path,
										item_path.last().unwrap().clone()
									))
									.unwrap(),
									itemdata,
								);
							}
						}
					}

					EngineResponse::GetSuccessFolder {
						folder: folder_data,
						children,
					}
				}
			} else {
				EngineResponse::InternalError(String::from("path is not a folder nor a document"))
			}
		}
	}

	fn new_for_tests() -> Self {
		let tempdir = std::path::PathBuf::from(tempfile::tempdir().unwrap().as_ref());

		std::fs::create_dir_all(tempdir.join("folder_a")).unwrap();
		std::fs::create_dir_all(tempdir.join("folder_b")).unwrap();
		std::fs::create_dir_all(tempdir.join("public").join("folder_c")).unwrap();

		let result = Self::new(EngineSettings {
			path: tempdir.clone(),
		});

		std::fs::write(
			tempdir.join("folder_a").join("document.txt"),
			b"My Document Content Here (folder a)",
		)
		.unwrap();
		std::fs::write(
			tempdir.join("folder_a").join("document.txt.itemdata.toml"),
			toml::to_string_pretty(&Item::Document {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
				content: None,
				content_type: Some("text/html".into()),
			})
			.unwrap(),
		)
		.unwrap();

		std::fs::write(
			tempdir.join("folder_b").join("document.txt"),
			b"My Document Content Here (folder b)",
		)
		.unwrap();
		std::fs::write(
			tempdir.join("folder_b").join("document.txt.itemdata.toml"),
			toml::to_string_pretty(&Item::Document {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
				content: None,
				content_type: Some("text/html".into()),
			})
			.unwrap(),
		)
		.unwrap();

		std::fs::write(
			tempdir.join("folder_b").join("other_document.txt"),
			b"My Other Document Content Here (folder b)",
		)
		.unwrap();
		std::fs::write(
			tempdir
				.join("folder_b")
				.join("other_document.txt.itemdata.toml"),
			toml::to_string_pretty(&Item::Document {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
				content: None,
				content_type: Some("text/html".into()),
			})
			.unwrap(),
		)
		.unwrap();

		std::fs::write(
			tempdir.join("folder_a").join(".folder.itemdata.toml"),
			toml::to_string_pretty(&Item::Folder {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
			})
			.unwrap(),
		)
		.unwrap();

		std::fs::write(
			tempdir.join("folder_b").join(".folder.itemdata.toml"),
			toml::to_string_pretty(&Item::Folder {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
			})
			.unwrap(),
		)
		.unwrap();

		std::fs::write(
			tempdir.join("public").join("folder_c").join("document.txt"),
			b"My Document Content Here (folder c)",
		)
		.unwrap();
		std::fs::write(
			tempdir
				.join("public")
				.join("folder_c")
				.join("document.txt.itemdata.toml"),
			toml::to_string_pretty(&Item::Document {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
				content: None,
				content_type: Some("text/html".into()),
			})
			.unwrap(),
		)
		.unwrap();

		std::fs::write(
			tempdir
				.join("public")
				.join("folder_c")
				.join(".folder.itemdata.toml"),
			toml::to_string_pretty(&Item::Folder {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
			})
			.unwrap(),
		)
		.unwrap();

		std::fs::write(
			tempdir.join("public").join(".folder.itemdata.toml"),
			toml::to_string_pretty(&Item::Folder {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
			})
			.unwrap(),
		)
		.unwrap();

		std::fs::write(
			tempdir.join("document.txt"),
			b"My Document Content Here (root)",
		)
		.unwrap();
		std::fs::write(
			tempdir.join("document.txt.itemdata.toml"),
			toml::to_string_pretty(&Item::Document {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
				content: None,
				content_type: Some("text/html".into()),
			})
			.unwrap(),
		)
		.unwrap();

		std::fs::write(
			tempdir.join(".folder.itemdata.toml"),
			toml::to_string_pretty(&Item::Folder {
				etag: Some(format!("{}", uuid::Uuid::new_v4()).into()),
				last_modified: Some(time::OffsetDateTime::now_utc().into()),
			})
			.unwrap(),
		)
		.unwrap();

		result
	}

	fn root_for_tests(&self) -> BTreeMap<Path, Item> {
		let mut result = list_folder(self.root_path.clone(), std::path::PathBuf::from(""));

		result.insert(
			pontus_onyx::item::ROOT_PATH.clone(),
			toml::from_str(
				&std::fs::read_to_string(self.root_path.join(format!(
					"{}",
					pontus_onyx::item::ROOT_PATH.as_datafile(".itemdata.toml")
				)))
				.unwrap(),
			)
			.unwrap(),
		);

		result
	}
}

fn list_folder(
	root_path: std::path::PathBuf,
	subfolder: std::path::PathBuf,
) -> BTreeMap<Path, Item> {
	let mut children = BTreeMap::new();

	for el in std::fs::read_dir(&root_path.join(&subfolder)).unwrap() {
		let entry = el.unwrap();

		if !entry.file_name().to_str().unwrap().contains(".itemdata.") {
			let item_path = pontus_onyx::item::Path::try_from(
				format!(
					"{}{}",
					entry.path().display(),
					if entry.metadata().unwrap().is_dir() {
						"/"
					} else {
						""
					}
				)
				.replace('\\', "/"),
			)
			.unwrap();

			let item_name = format!("{}", item_path.last().unwrap());

			let itemdata: Item = toml::from_str(
				&std::fs::read_to_string(format!("{}", item_path.as_datafile(".itemdata.toml")))
					.unwrap(),
			)
			.unwrap();
			let itemdata = if entry.metadata().unwrap().is_dir() {
				children.append(&mut list_folder(
					root_path.clone(),
					subfolder.join(&item_name),
				));

				itemdata
			} else {
				itemdata.content(std::fs::read(format!("{}", item_path)).unwrap())
			};

			children.insert(
				pontus_onyx::item::Path::try_from(format!("{}{}", subfolder.display(), &item_name))
					.unwrap(),
				itemdata,
			);
		}
	}

	children
}

#[cfg(test)]
mod generic_tests;
