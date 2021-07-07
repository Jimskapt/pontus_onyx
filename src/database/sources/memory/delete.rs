pub fn delete(
	root_item: &mut crate::Item,
	path: &std::path::Path,
	if_match: &crate::Etag,
) -> Result<crate::Etag, Box<dyn std::any::Any>> {
	if path.ends_with("/") {
		return Err(Box::new(DeleteError::DoesNotWorksForFolders));
	}

	let mut cumulated_path = std::path::PathBuf::new();
	for path_part in path {
		cumulated_path = cumulated_path.join(path_part);
		if let Err(error) = crate::database::utils::is_ok(path_part.to_str().unwrap()) {
			return Err(Box::new(DeleteError::IncorrectItemName {
				item_path: cumulated_path,
				error,
			}));
		}
	}

	cumulated_path = std::path::PathBuf::new();
	for path_part in path {
		cumulated_path = cumulated_path.join(path_part);
		if root_item.get_child(&cumulated_path).is_none() {
			return Err(Box::new(DeleteError::NotFound {
				item_path: cumulated_path,
			}));
		}
	}

	let parent_path = path.parent().unwrap();
	match root_item.get_child_mut(parent_path) {
		Some(crate::Item::Folder {
			content: Some(parent_content),
			..
		}) => match parent_content.get_mut(path.file_name().unwrap().to_str().unwrap()) {
			Some(found_item) => match &**found_item {
				crate::Item::Document {
					etag: found_etag, ..
				} => {
					if !if_match.is_empty() && if_match != found_etag {
						return Err(Box::new(DeleteError::NoIfMatch {
							item_path: path.to_path_buf(),
							search: if_match.clone(),
							found: found_etag.clone(),
						}));
					}
					let old_etag = found_etag.clone();

					parent_content.remove(path.file_name().unwrap().to_str().unwrap());

					{
						let parents = {
							let ancestors = path.ancestors();
							let paths: Vec<&std::path::Path> = ancestors.into_iter().collect();
							paths
						};

						for path_part in parents {
							if let Some(crate::Item::Folder {
								content: Some(parent_content),
								etag,
							}) = root_item.get_child_mut(path_part)
							{
								let mut to_delete = vec![];
								for (child_name, child_item) in &*parent_content {
									if let crate::Item::Folder {
										content: Some(child_content),
										..
									} = &**child_item
									{
										if child_content.is_empty() {
											to_delete.push(child_name.clone());
										}
									}
								}

								for child_name in to_delete {
									parent_content.remove(&child_name);
								}

								*etag = crate::Etag::new();
							}
						}
					}

					return Ok(old_etag);
				}
				crate::Item::Folder { .. } => {
					return Err(Box::new(DeleteError::DoesNotWorksForFolders));
				}
			},
			None => {
				return Err(Box::new(DeleteError::NotFound {
					item_path: path.to_path_buf(),
				}));
			}
		},
		Some(crate::Item::Folder { content: None, .. }) => {
			return Err(Box::new(DeleteError::NoContentInside {
				item_path: parent_path.to_path_buf(),
			}));
		}
		Some(crate::Item::Document { .. }) => {
			return Err(Box::new(DeleteError::Conflict {
				item_path: parent_path.to_path_buf(),
			}));
		}
		None => {
			return Err(Box::new(DeleteError::NotFound {
				item_path: parent_path.to_path_buf(),
			}));
		}
	}
}

#[derive(Debug, PartialEq)]
pub enum DeleteError {
	Conflict {
		item_path: std::path::PathBuf,
	},
	DoesNotWorksForFolders,
	NotFound {
		item_path: std::path::PathBuf,
	},
	NoContentInside {
		item_path: std::path::PathBuf,
	},
	IncorrectItemName {
		item_path: std::path::PathBuf,
		error: String,
	},
	NoIfMatch {
		item_path: std::path::PathBuf,
		search: crate::Etag,
		found: crate::Etag,
	},
}
impl std::fmt::Display for DeleteError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::Conflict{item_path} => f.write_fmt(format_args!("name conflict between folder and file on the path `{}`", item_path.to_string_lossy())),
			Self::DoesNotWorksForFolders => f.write_str("this method does not works on folders"),
			Self::NotFound{item_path} => f.write_fmt(format_args!("path not found : `{}`", item_path.to_string_lossy())),
			Self::NoContentInside{item_path} => f.write_fmt(format_args!("no content found in `{}`", item_path.to_string_lossy())),
			Self::IncorrectItemName{item_path, error} => f.write_fmt(format_args!("the path `{}` is incorrect, because {}", item_path.to_string_lossy(), error)),
			Self::NoIfMatch{item_path, search, found} => f.write_fmt(format_args!("the requested `{}` etag (through `IfMatch`) for `{}` was not found, found `{}` instead", search, item_path.to_string_lossy(), found)),
		}
	}
}
impl std::error::Error for DeleteError {}
impl crate::database::Error for DeleteError {
	fn to_response(&self, origin: &str, should_have_body: bool) -> actix_web::HttpResponse {
		match self {
			Self::Conflict { item_path: _ } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::DELETE,
				actix_web::http::StatusCode::CONFLICT,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::DoesNotWorksForFolders => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::DELETE,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::NotFound { item_path: _ } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::DELETE,
				actix_web::http::StatusCode::NOT_FOUND,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::NoContentInside { item_path: _ } => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::DELETE,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::IncorrectItemName {
				item_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::DELETE,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::NoIfMatch {
				item_path: _,
				search: _,
				found: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::DELETE,
				actix_web::http::StatusCode::PRECONDITION_FAILED,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(non_snake_case)]

	use super::{delete, DeleteError};

	// TODO : test last_modified

	fn build_test_db() -> (
		crate::Item,
		crate::Etag,
		crate::Etag,
		crate::Etag,
		crate::Etag,
		crate::Etag,
		crate::Etag,
	) {
		let root = crate::Item::new_folder(vec![
			(
				"A",
				crate::Item::new_folder(vec![
					(
						"AA",
						crate::Item::new_folder(vec![(
							"AAA",
							crate::Item::new_folder(vec![(
								"AAAA",
								crate::Item::new_doc(b"AAAA", "text/plain"),
							)]),
						)]),
					),
					("AB", crate::Item::new_doc(b"AB", "text/plain")),
				]),
			),
			(
				"public",
				crate::Item::new_folder(vec![(
					"C",
					crate::Item::new_folder(vec![(
						"CC",
						crate::Item::new_folder(vec![(
							"CCC",
							crate::Item::new_doc(b"CCC", "text/plain"),
						)]),
					)]),
				)]),
			),
		]);

		if let crate::Item::Folder {
			etag: root_etag,
			content: Some(content),
		} = &root
		{
			if let crate::Item::Folder {
				etag: A_etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				let AB_etag =
					if let crate::Item::Document { etag, .. } = &**content.get("AB").unwrap() {
						etag
					} else {
						panic!();
					};

				if let crate::Item::Folder {
					etag: AA_etag,
					content: Some(content),
				} = &**content.get("AA").unwrap()
				{
					if let crate::Item::Folder {
						etag: AAA_etag,
						content: Some(content),
					} = &**content.get("AAA").unwrap()
					{
						if let crate::Item::Document {
							etag: AAAA_etag, ..
						} = &**content.get("AAAA").unwrap()
						{
							return (
								root.clone(),
								root_etag.clone(),
								A_etag.clone(),
								AA_etag.clone(),
								AAA_etag.clone(),
								AAAA_etag.clone(),
								AB_etag.clone(),
							);
						} else {
							panic!();
						}
					} else {
						panic!();
					}
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	}

	#[test]
	fn simple_delete_on_not_existing() {
		let mut root = crate::Item::new_folder(vec![]);
		let root_etag = root.get_etag().clone();

		assert_eq!(
			*delete(
				&mut root,
				&std::path::PathBuf::from("A/AA/AAA/AAAA"),
				&crate::Etag::from(""),
			)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
			DeleteError::NotFound {
				item_path: std::path::PathBuf::from("A/")
			}
		);

		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_eq!(etag, root_etag);
			assert!(content.is_empty());
		} else {
			panic!();
		}
	}

	#[test]
	fn simple_delete_on_existing() {
		let (mut root, root_etag, A_etag, _, _, AAAA_etag, AB_etag) = build_test_db();

		let old_AAAA_etag = delete(
			&mut root,
			&std::path::PathBuf::from("A/AA/AAA/AAAA"),
			&crate::Etag::from(""),
		)
		.unwrap();

		assert_eq!(AAAA_etag, old_AAAA_etag);

		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_ne!(etag, root_etag);
			assert!(!content.is_empty());

			if let crate::Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_ne!(etag, &A_etag);
				assert!(!content.is_empty());

				assert_eq!(content.get("AA"), None);

				if let crate::Item::Document { etag, .. } = &**content.get("AB").unwrap() {
					assert_eq!(etag, &AB_etag);
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	}

	#[test]
	fn does_not_works_for_folders() {
		let (mut root, root_etag, A_etag, AA_etag, AAA_etag, AAAA_etag, _) = build_test_db();

		assert_eq!(
			*delete(
				&mut root,
				&std::path::PathBuf::from("A/AA/"),
				&crate::Etag::from(""),
			)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
			DeleteError::DoesNotWorksForFolders,
		);

		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = &root
		{
			assert_eq!(etag, &root_etag);
			if let crate::Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_eq!(etag, &A_etag);
				if let crate::Item::Folder {
					etag,
					content: Some(content),
				} = &**content.get("AA").unwrap()
				{
					assert_eq!(etag, &AA_etag);
					if let crate::Item::Folder {
						etag,
						content: Some(content),
					} = &**content.get("AAA").unwrap()
					{
						assert_eq!(etag, &AAA_etag);
						if let crate::Item::Document { etag, .. } = &**content.get("AAAA").unwrap()
						{
							assert_eq!(etag, &AAAA_etag);
						} else {
							panic!();
						}
					} else {
						panic!();
					}
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	}

	#[test]
	fn delete_with_if_match_not_found() {
		let (mut root, root_etag, A_etag, AA_etag, AAA_etag, AAAA_etag, _) = build_test_db();

		assert_eq!(
			*delete(
				&mut root,
				&std::path::PathBuf::from("A/AA/AAA/AAAA"),
				&crate::Etag::from("OTHER_ETAG"),
			)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
			DeleteError::NoIfMatch {
				item_path: std::path::PathBuf::from("A/AA/AAA/AAAA"),
				found: AAAA_etag.clone(),
				search: crate::Etag::from("OTHER_ETAG")
			}
		);

		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = &root
		{
			assert_eq!(etag, &root_etag);
			if let crate::Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_eq!(etag, &A_etag);
				if let crate::Item::Folder {
					etag,
					content: Some(content),
				} = &**content.get("AA").unwrap()
				{
					assert_eq!(etag, &AA_etag);
					if let crate::Item::Folder {
						etag,
						content: Some(content),
					} = &**content.get("AAA").unwrap()
					{
						assert_eq!(etag, &AAA_etag);
						if let crate::Item::Document { etag, .. } = &**content.get("AAAA").unwrap()
						{
							assert_eq!(etag, &AAAA_etag);
						} else {
							panic!();
						}
					} else {
						panic!();
					}
				} else {
					panic!();
				}
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	}

	#[test]
	fn delete_with_if_match_found() {
		let (mut root, root_etag, A_etag, _, _, AAAA_etag, _) = build_test_db();

		let old_AAAA_etag = delete(
			&mut root,
			&std::path::PathBuf::from("A/AA/AAA/AAAA"),
			&AAAA_etag,
		)
		.unwrap();

		assert_eq!(old_AAAA_etag, AAAA_etag);

		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = &root
		{
			assert_ne!(etag, &root_etag);
			assert!(!content.is_empty());

			if let crate::Item::Folder {
				etag,
				content: Some(content),
			} = &**content.get("A").unwrap()
			{
				assert_ne!(etag, &A_etag);
				assert!(!content.is_empty());

				assert_eq!(content.get("AA"), None);
				assert!(content.get("AB").is_some());
			} else {
				panic!();
			}
		} else {
			panic!();
		}
	}

	#[test]
	fn delete_in_public() {
		let (mut root, root_etag, _, _, _, _, _) = build_test_db();

		delete(
			&mut root,
			&std::path::PathBuf::from("public/C/CC/CCC"),
			&crate::Etag::from(""),
		)
		.unwrap();

		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = &root
		{
			assert_ne!(etag, &root_etag);
			assert!(!content.is_empty());

			assert!(content.get("A").is_some());
			assert_eq!(content.get("public"), None);
		} else {
			panic!();
		}
	}

	#[test]
	fn delete_in_incorrect_path() {
		let mut root = crate::Item::new_folder(vec![]);
		let root_etag = root.get_etag().clone();

		assert_eq!(
			*delete(
				&mut root,
				&std::path::PathBuf::from("A/../AA"),
				&crate::Etag::from(""),
			)
			.unwrap_err()
			.downcast::<DeleteError>()
			.unwrap(),
			DeleteError::IncorrectItemName {
				item_path: std::path::PathBuf::from("A/../"),
				error: String::from("`..` is not allowed")
			}
		);

		if let crate::Item::Folder {
			etag,
			content: Some(content),
		} = root
		{
			assert_eq!(etag, root_etag);
			assert!(content.is_empty());
		} else {
			panic!();
		}
	}
}