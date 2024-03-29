use std::sync::{Arc, Mutex};

/*
TODO :
	Unless [KERBEROS] is used (see section 10 below), all other
	requests SHOULD present a bearer token with sufficient access scope,
	using a header of the following form (no double quotes here):
		Authorization: Bearer <access_token>
*/
#[actix_web::put("/storage/{requested_item:.*}")]
pub async fn put_item(
	mut request_payload: actix_web::web::Payload,
	request: actix_web::HttpRequest,
	path: actix_web::web::Path<String>,
	database: actix_web::web::Data<std::sync::Arc<std::sync::Mutex<crate::database::Database>>>,
	logger: actix_web::web::Data<Arc<Mutex<charlie_buffalo::Logger>>>,
	dbevent_sender: actix_web::web::Data<std::sync::mpsc::Sender<crate::http_server::DbEvent>>,
	access_tokens: actix_web::web::Data<
		std::sync::Arc<std::sync::Mutex<Vec<crate::http_server::AccessBearer>>>,
	>,
) -> impl actix_web::Responder {
	let mut content = actix_web::web::BytesMut::new();
	while let Some(request_body) = futures::StreamExt::next(&mut request_payload).await {
		let request_body = request_body.unwrap();
		content.extend_from_slice(&request_body);
	}
	let content = content.freeze();

	let content_type = request.headers().get("content-type");

	// TODO : check security issue about this ?
	let all_origins = actix_web::http::header::HeaderValue::from_bytes(b"*").unwrap();
	let origin = request
		.headers()
		.get(actix_web::http::header::ORIGIN)
		.unwrap_or(&all_origins)
		.to_str()
		.unwrap();

	if content_type.is_none() {
		return crate::database::build_http_json_response(
			origin,
			request.method(),
			actix_web::http::StatusCode::BAD_REQUEST,
			None,
			None,
			Some(String::from("missing content-type HTTP header")),
			true,
		);
	}

	let local_path = crate::item::ItemPath::from(path.into_inner().as_str());

	let user = match request
		.headers()
		.get(actix_web::http::header::AUTHORIZATION)
	{
		Some(token) => {
			let token = match token.to_str().unwrap_or_default().strip_prefix("Bearer ") {
				Some(token) => token,
				None => token.to_str().unwrap_or_default(),
			};

			match access_tokens
				.lock()
				.unwrap()
				.iter()
				.find(|bearer| bearer.get_name() == token)
			{
				Some(bearer) => String::from(bearer.get_username()),
				None => String::from("Unknown"),
			}
		}
		None => String::from("Unknown"),
	};

	match database.lock().unwrap().put(
		&local_path,
		crate::item::Item::Document {
			etag: crate::item::Etag::from(""),
			content: Some(content.to_vec()),
			content_type: crate::item::ContentType::from(content_type.unwrap().to_str().unwrap()),
			last_modified: Some(time::OffsetDateTime::now_utc()),
		},
		super::convert_actix_if_match(&request)
			.first()
			.unwrap_or(&crate::item::Etag::from("")),
		&super::convert_actix_if_none_match(&request)
			.iter()
			.collect::<Vec<&crate::item::Etag>>(),
	) {
		crate::database::PutResult::Created(new_etag, last_modified) => {
			dbevent_sender
				.send(crate::http_server::DbEvent {
					id: ulid::Ulid::new().to_string(),
					method: crate::http_server::DbEventMethod::Create,
					date: last_modified,
					path: String::from("/storage/") + &local_path.to_string(),
					etag: new_etag.clone(),
					user,
					dbversion: String::from(env!("CARGO_PKG_VERSION")),
				})
				.ok();

			return crate::database::build_http_json_response(
				origin,
				request.method(),
				actix_web::http::StatusCode::CREATED,
				Some(new_etag),
				Some(last_modified),
				None,
				true,
			);
		}
		crate::database::PutResult::Updated(new_etag, last_modified) => {
			dbevent_sender
				.send(crate::http_server::DbEvent {
					id: ulid::Ulid::new().to_string(),
					method: crate::http_server::DbEventMethod::Update,
					date: last_modified,
					path: String::from("/storage/") + &local_path.to_string(),
					etag: new_etag.clone(),
					user,
					dbversion: String::from(env!("CARGO_PKG_VERSION")),
				})
				.ok();

			return crate::database::build_http_json_response(
				origin,
				request.method(),
				actix_web::http::StatusCode::OK,
				Some(new_etag),
				Some(last_modified),
				None,
				true,
			);
		}
		crate::database::PutResult::Err(e) => {
			if e.is::<crate::database::sources::memory::PutError>() {
				crate::database::Error::to_response(
					&*e.downcast::<crate::database::sources::memory::PutError>()
						.unwrap(),
					origin,
					true,
				)
			} else if e.is::<crate::database::sources::folder::PutError>() {
				crate::database::Error::to_response(
					&*e.downcast::<crate::database::sources::folder::PutError>()
						.unwrap(),
					origin,
					true,
				)
			} else {
				logger.lock().unwrap().push(
					vec![
						(String::from("level"), String::from("ERROR")),
						(String::from("module"), String::from("https?")),
						(String::from("method"), String::from("PUT")),
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

#[cfg(test)]
mod tests;
