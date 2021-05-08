#![allow(clippy::needless_return)]

extern crate pontus_onyx;

#[cfg(feature = "server")]
mod http_server;

/*
TODO : continue to :
	https://datatracker.ietf.org/doc/html/draft-dejong-remotestorage-16
		"12. Example wire transcripts"
*/

// TODO : anti brute-force for auth & /public/

#[cfg(feature = "server")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
	std::env::set_var(
		"RUST_LOG",
		"actix_example=debug,actix_web=debug,actix_http=debug,actix_service=debug",
	);
	env_logger::init();

	println!("starting to listen to http://localhost:7541/");

	let database = std::sync::Arc::new(std::sync::Mutex::new(
		pontus_onyx::Database::from_item_folder(pontus_onyx::Item::new_folder(vec![])).unwrap(),
	));

	let oauth_form_tokens: std::sync::Arc<
		std::sync::Mutex<Vec<crate::http_server::OauthFormToken>>,
	> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));

	let access_tokens: std::sync::Arc<std::sync::Mutex<Vec<crate::http_server::AccessBearer>>> =
		std::sync::Arc::new(std::sync::Mutex::new(vec![]));

	let mut users = crate::http_server::Users::new();
	users.insert("todo", "todo");
	let users: std::sync::Arc<std::sync::Mutex<crate::http_server::Users>> =
		std::sync::Arc::new(std::sync::Mutex::new(users));

	actix_web::HttpServer::new(move || {
		actix_web::App::new()
			.data(database.clone())
			.data(oauth_form_tokens.clone())
			.data(access_tokens.clone())
			.data(users.clone())
			.wrap(http_server::Auth {})
			.wrap(actix_web::middleware::Logger::default())
			.service(http_server::favicon)
			.service(http_server::get_oauth)
			.service(http_server::post_oauth)
			.service(http_server::webfinger_handle)
			.service(http_server::get_item)
			.service(http_server::head_item)
			.service(http_server::options_item)
			.service(http_server::put_item)
			.service(http_server::delete_item)
			.service(http_server::remotestoragesvg)
			.service(http_server::index)
	})
	.bind("localhost:7541")? // TODO : HTTPS
	.run()
	.await
}

/*
TODO ?
	Servers MAY support Content-Range headers [RANGE] on GET requests,
	but whether or not they do SHOULD be announced both through the
	"http://tools.ietf.org/html/rfc7233" option mentioned below in
	section 10 and through the HTTP 'Accept-Ranges' response header.
*/

/*
TODO :
* 401 for all requests that require a valid bearer token and
		where no valid one was sent (see also [BEARER, section
		3.1]),
* 403 for all requests that have insufficient scope, e.g.
		accessing a <module> for which no scope was obtained, or
		accessing data outside the user's <storage_root>,
* 413 if the payload is too large, e.g. when the server has a
		maximum upload size for documents
* 414 if the request URI is too long,
* 416 if Range requests are supported by the server and the Range
		request can not be satisfied,
* 429 if the client makes too frequent requests or is suspected
		of malicious activity,
* 4xx for all malformed requests, e.g. reserved characters in the
		path [URI, section 2.2], as well as for all PUT and DELETE
		requests to folders,
* 507 in case the account is over its storage quota,
*/
/*
TODO :
	All responses MUST carry CORS headers [CORS].
*/
/*
TODO :
	A "http://remotestorage.io/spec/web-authoring" property has been
	proposed with a string value of the fully qualified domain name to
	which web authoring content is published if the server supports web
	authoring as per [AUTHORING]. Note that this extension is a breaking
	extension in the sense that it divides users into "haves", whose
	remoteStorage accounts allow them to author web content, and
	"have-nots", whose remoteStorage account does not support this
	functionality.
*/
/*
TODO :
	The server MAY expire bearer tokens, and MAY require the user to
	register applications as OAuth clients before first use; if no
	client registration is required, the server MUST ignore the value of
	the client_id parameter in favor of relying on the origin of the
	redirect_uri parameter for unique client identification. See section
	4 of [ORIGIN] for computing the origin.
*/
/*
TODO :
	11. Storage-first bearer token issuance

	To request that the application connects to the user account
	<account> ' ' <host>, providers MAY redirect to applications with a
	'remotestorage' field in the URL fragment, with the user account as
	value.

	The appplication MUST make sure this request is intended by the
	user. It SHOULD ask for confirmation from the user whether they want
	to connect to the given provider account. After confirmation, it
	SHOULD connect to the given provider account, as defined in Section
	10.

	If the 'remotestorage' field exists in the URL fragment, the
	application SHOULD ignore any other parameters such as
	'access_token' or 'state'
*/

#[cfg(not(feature = "server"))]
fn main() {
	eprintln!(r#"WARNING : please build this binary at least with `--features "server"`"#);
}
