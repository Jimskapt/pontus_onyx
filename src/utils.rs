#[derive(serde::Serialize)]
struct JsonResponse {
	http_code: u16,
	#[serde(skip_serializing_if = "Option::is_none")]
	http_description: Option<&'static str>,
	#[serde(rename = "ETag", skip_serializing_if = "Option::is_none")]
	etag: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	hint: Option<String>,
}

pub fn build_http_json_response(
	request_method: &actix_web::http::Method,
	code: actix_web::http::StatusCode,
	etag: Option<String>,
	hint: Option<String>,
	should_have_body: bool,
) -> actix_web::HttpResponse {
	let mut response = actix_web::HttpResponse::build(code);
	response.content_type("application/ld+json");
	if request_method == actix_web::http::Method::GET && code.is_success() {
		response.header("Cache-Control", "no-cache");
	}
	response.header("Access-Control-Allow-Origin", "*");

	let mut expose_headers = String::from("Content-Length, Content-Type");
	if etag.is_some() {
		expose_headers += ", ETag";
	}
	response.header("Access-Control-Expose-Headers", expose_headers);

	if let Some(etag) = &etag {
		response.header("ETag", etag.clone());
	}

	return if should_have_body {
		response.body(
			serde_json::to_string(&JsonResponse {
				http_code: code.as_u16(),
				http_description: code.canonical_reason(),
				etag,
				hint,
			})
			.unwrap(),
		)
	} else {
		response.finish()
	};
}