use std::sync::{Arc, Mutex};

#[actix_web::head("/storage/{requested_item:.*}")]
pub async fn head_item(
	path: actix_web::web::Path<String>,
	request: actix_web::HttpRequest,
	database: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<crate::database::Database>>>,
	logger: actix_web::web::Data<Arc<Mutex<charlie_buffalo::Logger>>>,
) -> impl actix_web::Responder {
	// TODO : check security issue about this ?
	let all_origins = actix_web::http::header::HeaderValue::from_bytes(b"*").unwrap();
	let origin = request
		.headers()
		.get(actix_web::http::header::ORIGIN)
		.unwrap_or(&all_origins)
		.to_str()
		.unwrap();

	let local_path = crate::item::ItemPath::from(path.into_inner().as_str());

	match database.lock().unwrap().get(
		&local_path,
		super::convert_actix_if_match(&request)
			.first()
			.unwrap_or(&crate::item::Etag::from("")),
		&super::convert_actix_if_none_match(&request)
			.iter()
			.collect::<Vec<&crate::item::Etag>>(),
	) {
		Ok(crate::item::Item::Document {
			etag,
			content_type,
			last_modified,
			..
		}) => {
			let etag: String = etag.into();
			let mut response = actix_web::HttpResponse::Ok();
			response.insert_header((actix_web::http::header::ETAG, etag));
			if let Some(last_modified) = last_modified {
				response.insert_header((
					actix_web::http::header::LAST_MODIFIED,
					last_modified
						.format(&time::format_description::well_known::Rfc2822)
						.unwrap_or_default(),
				));
			}
			response.insert_header((actix_web::http::header::CACHE_CONTROL, "no-cache"));
			response.insert_header((actix_web::http::header::ACCESS_CONTROL_ALLOW_ORIGIN, origin));
			response.insert_header((
				actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
				"Content-Length, Content-Type, Etag, Last-Modified",
			));

			if origin != "*" {
				response.insert_header((actix_web::http::header::VARY, "Origin"));
			}

			let content_type: String = content_type.into();
			response.content_type(content_type);

			return response.finish();
		}
		Ok(crate::item::Item::Folder {
			etag: folder_etag,
			content: Some(content),
		}) => {
			let mut items_result = serde_json::json!({});
			for (child_name, child) in content.iter().filter(|(_, e)| match &***e {
				crate::item::Item::Document { .. } => true,
				crate::item::Item::Folder {
					content: Some(content),
					..
				} => !content.is_empty(),
				crate::item::Item::Folder { content: None, .. } => todo!(),
			}) {
				match &**child {
					crate::item::Item::Folder { etag, .. } => {
						items_result[format!("{}/", child_name)] = serde_json::json!({
							"ETag": etag,
						});
					}
					crate::item::Item::Document {
						etag,
						content: Some(document_content),
						content_type,
						last_modified,
					} => {
						let child_name: String = child_name.clone();
						items_result[child_name] = serde_json::json!({
							"ETag": etag,
							"Content-Type": content_type,
							"Content-Length": document_content.len(),
							"Last-Modified": if let Some(last_modified) = last_modified {
								serde_json::Value::from(last_modified.format(&time::format_description::well_known::Rfc2822).unwrap_or_default())
							} else {
								serde_json::Value::Null
							},
						});
					}
					crate::item::Item::Document { content: None, .. } => {
						logger.lock().unwrap().push(
							vec![
								(String::from("level"), String::from("ERROR")),
								(String::from("module"), String::from("https?")),
								(String::from("method"), String::from("HEAD")),
								(String::from("path"), local_path.to_string()),
							],
							Some("document has no content"),
						);

						return crate::database::build_http_json_response(
							origin,
							request.method(),
							actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
							None,
							None,
							None,
							false,
						);
					}
				}
			}

			let folder_etag: String = folder_etag.into();
			let mut response = actix_web::HttpResponse::Ok();
			response.content_type("application/ld+json");
			response.insert_header((actix_web::http::header::ETAG, folder_etag));
			response.insert_header((actix_web::http::header::CACHE_CONTROL, "no-cache"));
			response.insert_header((actix_web::http::header::ACCESS_CONTROL_ALLOW_ORIGIN, origin));
			response.insert_header((
				actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
				"Content-Length, Content-Type, Etag, Last-Modified",
			));

			if origin != "*" {
				response.insert_header((actix_web::http::header::VARY, "Origin"));
			}

			return response.finish();
		}
		Ok(crate::item::Item::Folder { content: None, .. }) => {
			logger.lock().unwrap().push(
				vec![
					(String::from("level"), String::from("ERROR")),
					(String::from("module"), String::from("https?")),
					(String::from("method"), String::from("HEAD")),
					(String::from("path"), local_path.to_string()),
				],
				Some("folder has no content"),
			);

			crate::database::build_http_json_response(
				origin,
				request.method(),
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				None,
				false,
			)
		}
		Err(e) => {
			if e.is::<crate::database::sources::memory::GetError>() {
				crate::database::Error::to_response(
					&*e.downcast::<crate::database::sources::memory::GetError>()
						.unwrap(),
					origin,
					true,
				)
			} else if e.is::<crate::database::sources::folder::GetError>() {
				crate::database::Error::to_response(
					&*e.downcast::<crate::database::sources::folder::GetError>()
						.unwrap(),
					origin,
					true,
				)
			} else {
				logger.lock().unwrap().push(
					vec![
						(String::from("level"), String::from("ERROR")),
						(String::from("module"), String::from("https?")),
						(String::from("method"), String::from("HEAD")),
						(String::from("path"), local_path.to_string()),
					],
					Some(&format!("error from database : {e}")),
				);

				crate::database::build_http_json_response(
					origin,
					request.method(),
					actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
					None,
					None,
					None,
					true,
				)
			}
		}
	}
}
