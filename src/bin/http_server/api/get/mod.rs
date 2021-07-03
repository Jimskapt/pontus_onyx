#[actix_web::get("/storage/{requested_item:.*}")]
pub async fn get_item(
	path: actix_web::web::Path<String>,
	request: actix_web::web::HttpRequest,
	database: actix_web::web::Data<
		std::sync::Arc<std::sync::Mutex<pontus_onyx::database::Database>>,
	>,
) -> impl actix_web::Responder {
	// TODO : check security issue about this ?
	let all_origins = actix_web::http::HeaderValue::from_bytes(b"*").unwrap();
	let origin = request
		.headers()
		.get(actix_web::http::header::ORIGIN)
		.unwrap_or(&all_origins)
		.to_str()
		.unwrap();

	match database.lock().unwrap().get(
		&std::path::PathBuf::from(path.to_string()),
		super::convert_actix_if_match(&request)
			.first()
			.unwrap_or(&&pontus_onyx::Etag::from("")),
		&super::convert_actix_if_none_match(&request)
			.iter()
			.collect::<Vec<&pontus_onyx::Etag>>(),
	) {
		Ok(pontus_onyx::Item::Document {
			etag,
			content: Some(content),
			content_type,
			..
		}) => {
			let etag: String = etag.clone().into();
			let content_type: String = content_type.clone().into();

			let mut response = actix_web::HttpResponse::Ok();
			response.header(actix_web::http::header::ETAG, etag);
			response.header(actix_web::http::header::CACHE_CONTROL, "no-cache");
			response.header(actix_web::http::header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);

			if origin != "*" {
				response.header(actix_web::http::header::VARY, "Origin");
			}

			response.header(
				actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
				"Content-Length, Content-Type, Etag",
			);
			response.content_type(content_type);

			return response.body(content.clone());
		}
		Ok(pontus_onyx::Item::Folder {
			etag: folder_etag,
			content: Some(content),
		}) => {
			let mut items_result = serde_json::json!({});
			for (child_name, child) in content.iter().filter(|(_, e)| match &***e {
				pontus_onyx::Item::Document { .. } => true,
				pontus_onyx::Item::Folder {
					content: Some(content),
					..
				} => !content.is_empty(),
				pontus_onyx::Item::Folder { content: None, .. } => todo!(),
			}) {
				match &**child {
					pontus_onyx::Item::Folder { etag, .. } => {
						items_result[format!("{}/", child_name)] = serde_json::json!({
							"ETag": etag,
						});
					}
					pontus_onyx::Item::Document {
						etag,
						content: Some(document_content),
						content_type,
						last_modified,
					} => {
						let child_name: String = child_name.clone().into();
						items_result[child_name] = serde_json::json!({
							"ETag": etag,
							"Content-Type": content_type,
							"Content-Length": document_content.len(),
							"Last-Modified": last_modified.format(crate::http_server::RFC5322).to_string(),
						});
					}
					pontus_onyx::Item::Document { content: None, .. } => {
						return pontus_onyx::database::build_http_json_response(
							origin,
							request.method(),
							actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
							None,
							None,
							true,
						);
					}
				}
			}

			let folder_etag: String = folder_etag.clone().into();

			let mut response = actix_web::HttpResponse::Ok();
			response.content_type("application/ld+json");
			response.header(actix_web::http::header::ETAG, folder_etag);
			response.header(actix_web::http::header::CACHE_CONTROL, "no-cache");
			response.header(actix_web::http::header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);

			if origin != "*" {
				response.header(actix_web::http::header::VARY, "Origin");
			}

			response.header(
				actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
				"Content-Length, Content-Type, Etag",
			);

			return response.body(
				serde_json::json!({
					"@context": "http://remotestorage.io/spec/folder-description",
					"items": items_result,
				})
				.to_string(),
			);
		}
		Ok(pontus_onyx::Item::Document { content: None, .. }) => {
			return pontus_onyx::database::build_http_json_response(
				origin,
				request.method(),
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				true,
			);
		}
		Ok(pontus_onyx::Item::Folder { content: None, .. }) => {
			return pontus_onyx::database::build_http_json_response(
				origin,
				request.method(),
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				true,
			);
		}
		Err(e) => e.to_response(origin, true),
	}
}

#[cfg(test)]
mod tests;
