#[derive(serde::Deserialize)]
pub struct OauthGetQuery {
	redirect_uri: String,
	scope: String,
	client_id: String,
	response_type: String,
	auth_result: Option<String>,
}

#[actix_web::get("/oauth/{username}")]
pub async fn get_oauth(
	path: actix_web::web::Path<String>,
	query: actix_web::web::Query<OauthGetQuery>,
	request: actix_web::HttpRequest,
	form_tokens: actix_web::web::Data<
		std::sync::Arc<std::sync::Mutex<Vec<crate::http_server::middlewares::OauthFormToken>>>,
	>,
) -> impl actix_web::Responder {
	let username = path.into_inner();

	let mut response = actix_web::HttpResponse::build(actix_web::http::StatusCode::OK);

	// TODO : sanitize user data before printing it ?
	// TODO : do not panic if input data is incorrect (especially scopes)

	let scopes = pct_str::PctString::new(&query.scope)
		.unwrap()
		.decode()
		.split(' ')
		.map(|scope_string| {
			(std::convert::TryFrom::try_from(scope_string.trim())
				as Result<crate::scope::Scope, crate::scope::ScopeParsingError>)
				.unwrap()
		})
		.map(|scope| {
			if scope.module == "*" {
				format!(
					r#"{} on {}<br><i style="color:red;">{}<br>{}</i>"#,
					scope.right_type,
					r#"<strong style="color:red;">all modules</strong>"#,
					r#"It is maybe an security issue."#,
					r#"You should accept it only if you know why this client need it !"#,
				)
			} else {
				format!(
					r#"{} on <a href="../storage/{}/{}/">/storage/{}/{}/</a> and <a href="../storage/public/{}/{}/">/storage/public/{}/{}/</a>"#,
					scope.right_type, username, scope.module, username, scope.module, username, scope.module, username, scope.module
				)
			}
		})
		.fold(String::new(), |acc, scope| {
			format!("{}<li>{}</li>", acc, scope)
		});

	let ip = request.peer_addr().unwrap();
	let new_token = crate::http_server::middlewares::OauthFormToken::new(ip);

	let mut new_tokens: Vec<crate::http_server::middlewares::OauthFormToken> = form_tokens
		.lock()
		.unwrap()
		.iter()
		.filter(|e| !e.should_be_cleaned(&ip))
		.cloned()
		.collect();
	new_tokens.push(new_token.clone());

	*form_tokens.lock().unwrap() = new_tokens;

	return response.body(format!(
		r#"<!DOCTYPE html>
<html>
	<head>
		<meta charset="UTF-8">
		<meta http-equiv="X-UA-Compatible" content="IE=edge">
		<meta name="viewport" content="width=device-width, initial-scale=1.0">
		<title>{} : allow access ?</title>
	</head>
	<body>
		<h1>Allow access ?</h1>
		<p>You are on your account management for this database.</p>
		<p>The client : {}</p>
		<p>Request following access to this scope(s) : <ul>{}</ul></p>
		<form method="post" action="/oauth">
			<input type="hidden" name="client_id" value="{}">
			<input type="hidden" name="redirect_uri" value="{}">
			<input type="hidden" name="response_type" value="{}">
			<input type="hidden" name="scope" value="{}">
			<input type="hidden" name="username" value="{}">
			<input type="hidden" name="allow" value="Allow">
			<input type="hidden" name="token" value="{}">

			<p>If you agree to this request, please write your password :<br>
				Account : {}<br>
				Password : <input type="password" name="password" value="">
			</p>

			<p><i>If success, you will be directly redirected on this client, with credentials.</i></p>{}
			<input type="submit">
		</form>
	</body>
</html>"#,
		env!("CARGO_PKG_NAME"),
		query.client_id,
		scopes,
		query.client_id,
		pct_str::PctString::encode(
			pct_str::PctString::new(&query.redirect_uri)
				.unwrap()
				.decode()
				.chars(),
			pct_str::URIReserved
		),
		query.response_type,
		pct_str::PctString::encode(
			pct_str::PctString::new(&query.scope)
				.unwrap()
				.decode()
				.chars(),
			pct_str::URIReserved
		),
		pct_str::PctString::encode(
			pct_str::PctString::new(&username).unwrap().decode().chars(),
			pct_str::URIReserved
		),
		pct_str::PctString::encode(new_token.get_value().chars(), pct_str::URIReserved),
		pct_str::PctString::encode(
			pct_str::PctString::new(&username).unwrap().decode().chars(),
			pct_str::URIReserved
		),
		match &query.auth_result {
			Some(code) if code == "wrong_credentials" =>
				String::from(r#"<p class="error">Wrong credentials.</p>"#),
			Some(code) if code == "security_issue" => String::from(
				r#"<p class="error">There is an security issue, please try again.</p>"#
			),
			Some(code) => format!(r#"<p class="error">Unknown error : {}.</p>"#, code),
			None => String::new(),
		}
	));
}
