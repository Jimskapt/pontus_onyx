pub struct Response {
	pub request: crate::Request,
	pub status: ResponseStatus,
}

#[derive(Debug, PartialEq)]
pub enum ResponseStatus {
	Performed(crate::EngineResponse),
	Unallowed(crate::AccessError),
	NoIfMatch(crate::item::Etag),
	IfNoneMatch(crate::item::Etag),
	ContentNotChanged,
	NotSuitableForFolderItem,
	MissingItem,
	InternalError(String),
}

#[cfg(feature = "actix_server")]
async fn to_actix_response(internal_response: Response) -> actix_web::HttpResponse {
	match &internal_response.status {
		ResponseStatus::Performed(crate::EngineResponse::GetSuccessDocument(item)) => {
			let mut response = actix_web::HttpResponse::build(actix_web::http::StatusCode::OK);
			response.insert_header((actix_web::http::header::CACHE_CONTROL, "no-cache"));
			response.insert_header((
				actix_web::http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
				internal_response.request.origin.clone(),
			));

			if internal_response.request.origin != "*" {
				response.insert_header((actix_web::http::header::VARY, "Origin"));
			}

			match item {
				crate::item::Item::Document {
					etag,
					last_modified,
					content,
					content_type,
				} => {
					response.insert_header((
						actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
						"Content-Length, Content-Type, ETag, Last-Modified",
					));

					if let Some(content_type) = content_type {
						response.content_type(content_type.into_inner());
					}

					if let Some(etag) = etag {
						response.insert_header((actix_web::http::header::ETAG, etag.to_string()));
					}

					if let Some(last_modified) = last_modified {
						if let Ok(last_modified) = last_modified
							.into_inner()
							.format(&time::format_description::well_known::Rfc2822)
						{
							response.insert_header((
								actix_web::http::header::LAST_MODIFIED,
								last_modified,
							));
						}
					}

					// TODO : better error event handler than unwrap, or let actix deal with panic ?
					response.body::<Vec<u8>>(content.as_ref().unwrap().into_inner().to_vec())
				}
				crate::item::Item::Folder { .. } => {
					response.status(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
					response.body::<&[u8]>(br#"{"code":500, "error":"internal server error", "hint": "see server logs to understand"}"#)
				}
			}
		}
		ResponseStatus::Performed(crate::EngineResponse::GetSuccessFolder { folder, children }) => {
			let mut response = actix_web::HttpResponse::build(actix_web::http::StatusCode::OK);
			response.insert_header((actix_web::http::header::CACHE_CONTROL, "no-cache"));
			response.insert_header((
				actix_web::http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
				internal_response.request.origin.clone(),
			));

			if internal_response.request.origin != "*" {
				response.insert_header((actix_web::http::header::VARY, "Origin"));
			}

			match folder {
				crate::item::Item::Document { .. } => {
					response.status(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
					response.content_type("application/ld+json");
					response.body::<&[u8]>(br#"{"code":500, "error":"internal server error", "hint": "see server logs to understand"}"#)
				}
				crate::item::Item::Folder {
					etag,
					last_modified,
				} => {
					response.content_type("application/ld+json");

					response.insert_header((
						actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
						"Content-Length, Content-Type, ETag, Last-Modified",
					));

					if let Some(etag) = etag {
						response.insert_header((actix_web::http::header::ETAG, etag.to_string()));
					}

					if let Some(last_modified) = last_modified {
						if let Ok(last_modified) = last_modified
							.into_inner()
							.format(&time::format_description::well_known::Rfc2822)
						{
							response.insert_header((
								actix_web::http::header::LAST_MODIFIED,
								last_modified,
							));
						}
					}

					let mut items_result = serde_json::json!({});
					for (child_path, child) in children {
						if let Some(child_name) = child_path.last() {
							match child {
								crate::item::Item::Document {
									etag,
									content,
									content_type,
									last_modified,
								} => {
									items_result[format!("{}", child_name)] = serde_json::json!({
										"ETag": etag,
										"Content-Type": content_type,
										"Content-Length": content.as_ref().unwrap().into_inner().len(), // TODO : better error event handler, or let actix deal with panic ?
										"Last-Modified": if let Some(last_modified) = last_modified {
											serde_json::Value::from(last_modified.into_inner().format(&time::format_description::well_known::Rfc2822).unwrap_or_default())
										} else {
											serde_json::Value::Null
										},
									});
								}
								crate::item::Item::Folder { etag, .. } => {
									items_result[format!("{}", child_name)] = serde_json::json!({
										"ETag": etag,
									});
								}
							}
						}
					}

					return response.body(
						serde_json::json!({
							"@context": "http://remotestorage.io/spec/folder-description",
							"items": items_result,
						})
						.to_string(),
					);
				}
			}
		}
		ResponseStatus::Performed(crate::EngineResponse::CreateSuccess(etag, last_modified)) => {
			todo!()
		}
		ResponseStatus::Performed(crate::EngineResponse::UpdateSuccess(etag, last_modified)) => {
			todo!()
		}
		ResponseStatus::Performed(crate::EngineResponse::DeleteSuccess) => {
			todo!()
		}
		ResponseStatus::Performed(crate::EngineResponse::NotFound) => {
			todo!()
		}
		ResponseStatus::Performed(crate::EngineResponse::InternalError(error_details)) => {
			todo!()
		}
		ResponseStatus::Unallowed(error) => {
			todo!()
		}
		ResponseStatus::NoIfMatch(found) => {
			todo!()
		}
		ResponseStatus::IfNoneMatch(found) => {
			todo!()
		}
		ResponseStatus::ContentNotChanged => {
			todo!()
		}
		ResponseStatus::NotSuitableForFolderItem => {
			todo!()
		}
		ResponseStatus::MissingItem => {
			todo!()
		}
		ResponseStatus::InternalError(error) => {
			todo!()
		}
	}
}

#[cfg(feature = "actix_server")]
#[tokio::test]
async fn convert_actix_response() {
	todo!()
}
