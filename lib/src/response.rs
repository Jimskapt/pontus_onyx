pub struct Response {
	pub request: crate::Request,
	pub status: ResponseStatus,
}

#[derive(Debug, PartialEq)]
pub enum ResponseStatus {
	Performed(crate::EngineResponse),
	Unauthorized(crate::AccessError),
	NoIfMatch(crate::item::Etag),
	IfNoneMatch(crate::item::Etag),
	ContentNotChanged,
	NotSuitableForFolderItem,
	MissingRequestItem,
	InternalError(String),
}

#[cfg(feature = "actix_server")]
impl From<Response> for actix_web::HttpResponse {
	fn from(internal_response: Response) -> Self {
		let mut response =
			actix_web::HttpResponse::build(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
		response.insert_header((actix_web::http::header::CACHE_CONTROL, "no-cache"));
		response.insert_header((
			actix_web::http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
			internal_response.request.origin.clone(),
		));

		if internal_response.request.origin != "*" {
			response.insert_header((actix_web::http::header::VARY, "Origin"));
		}

		match &internal_response.status {
			ResponseStatus::Performed(crate::EngineResponse::GetSuccessDocument(item)) => {
				match item {
					crate::item::Item::Document {
						etag,
						last_modified,
						content,
						content_type,
					} => {
						let response_status = actix_web::http::StatusCode::OK;
						response.status(response_status);

						response.insert_header((
							actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
							"Content-Length, Content-Type, ETag, Last-Modified",
						));

						if let Some(content_type) = content_type {
							response.content_type(content_type.into_inner());
						}

						if let Some(etag) = etag {
							response
								.insert_header((actix_web::http::header::ETAG, etag.to_string()));
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

						if internal_response.request.method == crate::Method::Head {
							response.finish()
						} else {
							// TODO : better error event handler than unwrap, or let actix deal with panic ?
							response
								.body::<Vec<u8>>(content.as_ref().unwrap().into_inner().to_vec())
						}
					}
					crate::item::Item::Folder { .. } => {
						let response_status = actix_web::http::StatusCode::INTERNAL_SERVER_ERROR;
						response.status(response_status);
						response.content_type("application/ld+json");
						if internal_response.request.method == crate::Method::Head {
							response.finish()
						} else {
							response.body::<String>(
								serde_json::json!({
									"code": u16::from(response_status),
									"description": response_status.canonical_reason(),
									"hint": "see server logs to understand why"
								})
								.to_string(),
							)
						}
					}
				}
			}
			ResponseStatus::Performed(crate::EngineResponse::GetSuccessFolder {
				folder,
				children,
			}) => {
				match folder {
					crate::item::Item::Document { .. } => {
						let response_status = actix_web::http::StatusCode::INTERNAL_SERVER_ERROR;
						response.status(response_status);
						response.content_type("application/ld+json");
						if internal_response.request.method == crate::Method::Head {
							response.finish()
						} else {
							response.body::<String>(
								serde_json::json!({
									"code": u16::from(response_status),
									"description": response_status.canonical_reason(),
									"hint": "see server logs to understand why"
								})
								.to_string(),
							)
						}
					}
					crate::item::Item::Folder {
						etag,
						last_modified,
					} => {
						let response_status = actix_web::http::StatusCode::OK;
						response.status(response_status);

						response.content_type("application/ld+json");

						response.insert_header((
							actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
							"Content-Length, Content-Type, ETag, Last-Modified",
						));

						if let Some(etag) = etag {
							response
								.insert_header((actix_web::http::header::ETAG, etag.to_string()));
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

						if internal_response.request.method == crate::Method::Head {
							response.finish()
						} else {
							response.body(
								serde_json::json!({
									"@context": "http://remotestorage.io/spec/folder-description",
									"items": items_result,
								})
								.to_string(),
							)
						}
					}
				}
			}
			ResponseStatus::Performed(crate::EngineResponse::CreateSuccess(
				etag,
				last_modified,
			)) => {
				let response_status = actix_web::http::StatusCode::CREATED;
				response.status(response_status);

				response.content_type("application/ld+json");

				response.insert_header((
					actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
					"Content-Length, Content-Type, ETag, Last-Modified",
				));

				response.insert_header((actix_web::http::header::ETAG, etag.to_string()));

				if let Ok(last_modified) = last_modified
					.into_inner()
					.format(&time::format_description::well_known::Rfc2822)
				{
					response.insert_header((actix_web::http::header::LAST_MODIFIED, last_modified));
				}

				if internal_response.request.method == crate::Method::Head {
					response.finish()
				} else {
					response.body::<String>(
						serde_json::json!({
							"code": u16::from(response_status),
							"description": response_status.canonical_reason(),
						})
						.to_string(),
					)
				}
			}
			ResponseStatus::Performed(crate::EngineResponse::UpdateSuccess(
				etag,
				last_modified,
			)) => {
				let response_status = actix_web::http::StatusCode::OK;
				response.status(response_status);

				response.content_type("application/ld+json");

				response.insert_header((
					actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
					"Content-Length, Content-Type, ETag, Last-Modified",
				));

				response.insert_header((actix_web::http::header::ETAG, etag.to_string()));

				if let Ok(last_modified) = last_modified
					.into_inner()
					.format(&time::format_description::well_known::Rfc2822)
				{
					response.insert_header((actix_web::http::header::LAST_MODIFIED, last_modified));
				}

				if internal_response.request.method == crate::Method::Head {
					response.finish()
				} else {
					response.body::<String>(
						serde_json::json!({
							"code": u16::from(response_status),
							"description": response_status.canonical_reason(),
						})
						.to_string(),
					)
				}
			}
			ResponseStatus::Performed(crate::EngineResponse::DeleteSuccess) => {
				let response_status = actix_web::http::StatusCode::OK;
				response.status(response_status);

				response.content_type("application/ld+json");

				response.insert_header((
					actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
					"Content-Length, Content-Type",
				));

				if internal_response.request.method == crate::Method::Head {
					response.finish()
				} else {
					response.body::<String>(
						serde_json::json!({
							"code": u16::from(response_status),
							"description": response_status.canonical_reason(),
						})
						.to_string(),
					)
				}
			}
			ResponseStatus::Performed(crate::EngineResponse::NotFound) => {
				let response_status = actix_web::http::StatusCode::NOT_FOUND;
				response.status(response_status);

				response.content_type("application/ld+json");

				response.insert_header((
					actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
					"Content-Length, Content-Type",
				));

				if internal_response.request.method == crate::Method::Head {
					response.finish()
				} else {
					response.body::<String>(
						serde_json::json!({
							"code": u16::from(response_status),
							"description": response_status.canonical_reason(),
						})
						.to_string(),
					)
				}
			}
			ResponseStatus::Performed(crate::EngineResponse::InternalError(_)) => {
				let response_status = actix_web::http::StatusCode::INTERNAL_SERVER_ERROR;
				response.status(response_status);

				response.content_type("application/ld+json");

				response.insert_header((
					actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
					"Content-Length, Content-Type",
				));

				if internal_response.request.method == crate::Method::Head {
					response.finish()
				} else {
					response.body::<String>(
						serde_json::json!({
							"code": u16::from(response_status),
							"description": response_status.canonical_reason(),
						})
						.to_string(),
					)
				}
			}
			ResponseStatus::Unauthorized(_) => {
				let response_status = actix_web::http::StatusCode::UNAUTHORIZED;
				response.status(response_status);

				response.content_type("application/ld+json");

				response.insert_header((
					actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
					"Content-Length, Content-Type",
				));

				if internal_response.request.method == crate::Method::Head {
					response.finish()
				} else {
					response.body::<String>(
						serde_json::json!({
							"code": u16::from(response_status),
							"description": response_status.canonical_reason(),
						})
						.to_string(),
					)
				}
			}
			ResponseStatus::NoIfMatch(_) => {
				let response_status = actix_web::http::StatusCode::PRECONDITION_FAILED;
				response.status(response_status);

				response.content_type("application/ld+json");

				response.insert_header((
					actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
					"Content-Length, Content-Type",
				));

				if internal_response.request.method == crate::Method::Head {
					response.finish()
				} else {
					response.body::<String>(
						serde_json::json!({
							"code": u16::from(response_status),
							"description": response_status.canonical_reason(),
							"hint": "If-Match header has not matched"
						})
						.to_string(),
					)
				}
			}
			ResponseStatus::IfNoneMatch(_) => {
				let response_status = actix_web::http::StatusCode::PRECONDITION_FAILED;
				response.status(response_status);

				response.content_type("application/ld+json");

				response.insert_header((
					actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
					"Content-Length, Content-Type",
				));

				if internal_response.request.method == crate::Method::Head {
					response.finish()
				} else {
					response.body::<String>(
						serde_json::json!({
							"code": u16::from(response_status),
							"description": response_status.canonical_reason(),
							"hint": "If-None-Match header has matched"
						})
						.to_string(),
					)
				}
			}
			ResponseStatus::ContentNotChanged => {
				let response_status = actix_web::http::StatusCode::NOT_MODIFIED;
				response.status(response_status);

				response.content_type("application/ld+json");

				response.insert_header((
					actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
					"Content-Length, Content-Type",
				));

				if internal_response.request.method == crate::Method::Head {
					response.finish()
				} else {
					response.body::<String>(
						serde_json::json!({
							"code": u16::from(response_status),
							"description": response_status.canonical_reason(),
						})
						.to_string(),
					)
				}
			}
			ResponseStatus::NotSuitableForFolderItem => {
				let response_status = actix_web::http::StatusCode::BAD_REQUEST;
				response.status(response_status);

				response.content_type("application/ld+json");

				response.insert_header((
					actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
					"Content-Length, Content-Type",
				));

				if internal_response.request.method == crate::Method::Head {
					response.finish()
				} else {
					response.body::<String>(
						serde_json::json!({
							"code": u16::from(response_status),
							"description": response_status.canonical_reason(),
							"hint": "this request is not allowed on folders",
						})
						.to_string(),
					)
				}
			}
			ResponseStatus::MissingRequestItem => {
				let response_status = actix_web::http::StatusCode::BAD_REQUEST;
				response.status(response_status);

				response.content_type("application/ld+json");

				response.insert_header((
					actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
					"Content-Length, Content-Type",
				));

				if internal_response.request.method == crate::Method::Head {
					response.finish()
				} else {
					response.body::<String>(
						serde_json::json!({
							"code": u16::from(response_status),
							"description": response_status.canonical_reason(),
							"hint": "please provide payload/content in the request",
						})
						.to_string(),
					)
				}
			}
			ResponseStatus::InternalError(_) => {
				let response_status = actix_web::http::StatusCode::INTERNAL_SERVER_ERROR;
				response.status(response_status);

				response.content_type("application/ld+json");

				response.insert_header((
					actix_web::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
					"Content-Length, Content-Type",
				));

				if internal_response.request.method == crate::Method::Head {
					response.finish()
				} else {
					response.body::<String>(
						serde_json::json!({
							"code": u16::from(response_status),
							"description": response_status.canonical_reason(),
						})
						.to_string(),
					)
				}
			}
		}
	}
}
