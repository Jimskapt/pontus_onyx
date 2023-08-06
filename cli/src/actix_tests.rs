use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as AsyncMutex;

const USER: &str = "test_user";
const PASSWORD: &str = "test_password";

async fn build_test_server() -> (impl FnOnce(&mut actix_web::web::ServiceConfig), String) {
	let mut settings = crate::settings::Settings::default();
	settings.port = Some(6666);

	let temp_workspace = tempfile::tempdir().unwrap().into_path();
	println!("temporary workspace : {}", temp_workspace.display());
	settings.logfile_path = temp_workspace
		.join("server.log")
		.as_os_str()
		.to_str()
		.unwrap()
		.to_string();
	settings.userfile_path = temp_workspace
		.join("users.bin")
		.as_os_str()
		.to_str()
		.unwrap()
		.to_string();
	settings.data_path = temp_workspace
		.join("data")
		.as_os_str()
		.to_str()
		.unwrap()
		.to_string();

	let mut database = pontus_onyx::Database::new(
		<pontus_onyx_engine_filesystem::FileSystemEngine as pontus_onyx::Engine>::new(
			pontus_onyx_engine_filesystem::EngineSettings {
				path: settings.data_path.clone().into(),
			},
		),
	);

	database
		.create_user(USER, &mut String::from(PASSWORD))
		.unwrap();
	let token = database
		.generate_token(USER, &mut String::from(PASSWORD), "alpha:rw beta:r")
		.unwrap();

	// temporary admin :
	database
		.create_user("trdh8gb45sg6t", &mut String::from("56swefvrwsd3g96sgw"))
		.unwrap();
	let admin_token = database
		.generate_token(
			"trdh8gb45sg6t",
			&mut String::from("56swefvrwsd3g96sgw"),
			"*:rw",
		)
		.unwrap();

	// create assets :
	database
		.perform(
			pontus_onyx::Request::put(
				pontus_onyx::item::Path::try_from(format!("/alpha/documentA1.txt")).unwrap(),
				"test",
			)
			.token(admin_token.clone())
			.item(pontus_onyx::item::Item::Document {
				etag: None,
				last_modified: None,
				content: Some(b"documentA1".into()),
				content_type: Some("text/plain".into()),
			}),
		)
		.await
		.unwrap();
	database
		.perform(
			pontus_onyx::Request::put(
				pontus_onyx::item::Path::try_from(format!("/alpha/documentA2.txt")).unwrap(),
				"test",
			)
			.token(admin_token.clone())
			.item(pontus_onyx::item::Item::Document {
				etag: None,
				last_modified: None,
				content: Some(b"documentA2".into()),
				content_type: Some("text/plain".into()),
			}),
		)
		.await
		.unwrap();
	database
		.perform(
			pontus_onyx::Request::put(
				pontus_onyx::item::Path::try_from(format!("/alpha/sub_alpha/documentAA1.txt"))
					.unwrap(),
				"test",
			)
			.token(admin_token.clone())
			.item(pontus_onyx::item::Item::Document {
				etag: None,
				last_modified: None,
				content: Some(b"documentAA1".into()),
				content_type: Some("text/plain".into()),
			}),
		)
		.await
		.unwrap();
	database
		.perform(
			pontus_onyx::Request::put(
				pontus_onyx::item::Path::try_from(format!("/beta/documentB1.txt")).unwrap(),
				"test",
			)
			.token(admin_token.clone())
			.item(pontus_onyx::item::Item::Document {
				etag: None,
				last_modified: None,
				content: Some(b"documentB1".into()),
				content_type: Some("text/plain".into()),
			}),
		)
		.await
		.unwrap();

	let form_tokens = Arc::new(Mutex::new(vec![]));

	let program_state =
		crate::ProgramState::from(&settings, &std::path::PathBuf::from("/dev/null"));

	(
		crate::configure_server(
			settings,
			Arc::new(AsyncMutex::new(database)),
			Arc::new(Mutex::new(program_state)),
			form_tokens,
		),
		token.0,
	)
}

#[actix_web::test]
async fn get_index() {
	let (conf, _) = build_test_server().await;
	let app = actix_web::test::init_service(
		actix_web::App::new()
			.wrap(actix_web::middleware::Logger::default())
			.configure(conf),
	)
	.await;

	let request = actix_web::test::TestRequest::get().uri("/").to_request();
	let call_res = actix_web::test::call_service(&app, request).await;

	assert_eq!(call_res.status(), actix_web::http::StatusCode::OK);
}

#[actix_web::test]
async fn get_without_token() {
	let (conf, _) = build_test_server().await;
	let app = actix_web::test::init_service(
		actix_web::App::new()
			.wrap(actix_web::middleware::Logger::default())
			.configure(conf),
	)
	.await;

	let request = actix_web::test::TestRequest::get()
		.uri("/storage/")
		.to_request();
	let call_res = actix_web::test::call_service(&app, request).await;
	assert_eq!(call_res.status(), actix_web::http::StatusCode::UNAUTHORIZED);

	let request = actix_web::test::TestRequest::get()
		.uri(&format!("/storage/"))
		.to_request();
	let call_res = actix_web::test::call_service(&app, request).await;
	assert_eq!(call_res.status(), actix_web::http::StatusCode::UNAUTHORIZED);

	let request = actix_web::test::TestRequest::get()
		.uri(&format!("/storage/alpha/"))
		.to_request();
	let call_res = actix_web::test::call_service(&app, request).await;
	assert_eq!(call_res.status(), actix_web::http::StatusCode::UNAUTHORIZED);

	let request = actix_web::test::TestRequest::get()
		.uri(&format!("/storage/alpha/documentA1.txt"))
		.to_request();
	let call_res = actix_web::test::call_service(&app, request).await;
	assert_eq!(call_res.status(), actix_web::http::StatusCode::UNAUTHORIZED);

	let request = actix_web::test::TestRequest::get()
		.uri(&format!("/storage/beta/"))
		.to_request();
	let call_res = actix_web::test::call_service(&app, request).await;
	assert_eq!(call_res.status(), actix_web::http::StatusCode::UNAUTHORIZED);

	let request = actix_web::test::TestRequest::get()
		.uri(&format!("/storage/gamma/"))
		.to_request();
	let call_res = actix_web::test::call_service(&app, request).await;
	assert_eq!(call_res.status(), actix_web::http::StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn get_with_token() {
	let (conf, token) = build_test_server().await;
	let app = actix_web::test::init_service(
		actix_web::App::new()
			.wrap(actix_web::middleware::Logger::default())
			.configure(conf),
	)
	.await;

	let request = actix_web::test::TestRequest::get()
		.uri("/storage/")
		.insert_header(("Authorization", format!("Bearer {token}")))
		.to_request();
	let call_res = actix_web::test::call_service(&app, request).await;
	assert_eq!(call_res.status(), actix_web::http::StatusCode::UNAUTHORIZED);

	let request = actix_web::test::TestRequest::get()
		.uri(&format!("/storage/"))
		.insert_header(("Authorization", format!("Bearer {token}")))
		.to_request();
	let call_res = actix_web::test::call_service(&app, request).await;
	assert_eq!(call_res.status(), actix_web::http::StatusCode::UNAUTHORIZED);

	let request = actix_web::test::TestRequest::get()
		.uri(&format!("/storage/alpha/"))
		.insert_header(("Authorization", format!("Bearer {token}")))
		.to_request();
	let call_res = actix_web::test::call_service(&app, request).await;
	assert_eq!(call_res.status(), actix_web::http::StatusCode::OK);

	let request = actix_web::test::TestRequest::get()
		.uri(&format!("/storage/alpha/documentA1.txt"))
		.insert_header(("Authorization", format!("Bearer {token}")))
		.to_request();
	let call_res = actix_web::test::call_service(&app, request).await;
	assert_eq!(call_res.status(), actix_web::http::StatusCode::OK);

	let request = actix_web::test::TestRequest::get()
		.uri(&format!("/storage/beta/"))
		.insert_header(("Authorization", format!("Bearer {token}")))
		.to_request();
	let call_res = actix_web::test::call_service(&app, request).await;
	assert_eq!(call_res.status(), actix_web::http::StatusCode::OK);

	let request = actix_web::test::TestRequest::get()
		.uri(&format!("/storage/gamma/"))
		.insert_header(("Authorization", format!("Bearer {token}")))
		.to_request();
	let call_res = actix_web::test::call_service(&app, request).await;
	assert_eq!(call_res.status(), actix_web::http::StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn put_with_token() {
	let (conf, token) = build_test_server().await;
	let app = actix_web::test::init_service(
		actix_web::App::new()
			.wrap(actix_web::middleware::Logger::default())
			.configure(conf),
	)
	.await;

	let request = actix_web::test::TestRequest::put()
		.uri("/storage/alpha")
		.insert_header(("Authorization", format!("Bearer {token}")))
		.to_request();
	let call_res = actix_web::test::call_service(&app, request).await;
	assert_eq!(call_res.status(), actix_web::http::StatusCode::UNAUTHORIZED);

	let request = actix_web::test::TestRequest::put()
		.uri("/storage/alpha/documentA1.txt")
		.insert_header(("Authorization", format!("Bearer {token}")))
		.insert_header(("Content-Type", "text/plain"))
		.set_payload(b"documentA1".to_vec())
		.to_request();
	let call_res = actix_web::test::call_service(&app, request).await;
	assert_eq!(call_res.status(), actix_web::http::StatusCode::NOT_MODIFIED);

	let request = actix_web::test::TestRequest::put()
		.uri("/storage/alpha/documentA1.txt")
		.insert_header(("Authorization", format!("Bearer {token}")))
		.insert_header(("Content-Type", "text/plain"))
		.set_payload(b"documentA1-new".to_vec())
		.to_request();
	let call_res = actix_web::test::call_service(&app, request).await;
	assert_eq!(call_res.status(), actix_web::http::StatusCode::OK);

	let request = actix_web::test::TestRequest::put()
		.uri("/storage/alpha/documentA3.txt")
		.insert_header(("Authorization", format!("Bearer {token}")))
		.insert_header(("Content-Type", "text/plain"))
		.set_payload(b"documentA3".to_vec())
		.to_request();
	let call_res = actix_web::test::call_service(&app, request).await;
	assert_eq!(call_res.status(), actix_web::http::StatusCode::CREATED);

	let request = actix_web::test::TestRequest::put()
		.uri("/storage/alpha/sub_alpha/documentAA1.txt")
		.insert_header(("Authorization", format!("Bearer {token}")))
		.insert_header(("Content-Type", "text/plain"))
		.set_payload(b"documentAA1-new".to_vec())
		.to_request();
	let call_res = actix_web::test::call_service(&app, request).await;
	assert_eq!(call_res.status(), actix_web::http::StatusCode::OK);

	let request = actix_web::test::TestRequest::put()
		.uri("/storage/beta/documentB1.txt")
		.insert_header(("Authorization", format!("Bearer {token}")))
		.insert_header(("Content-Type", "text/plain"))
		.set_payload(b"documentB1-new".to_vec())
		.to_request();
	let call_res = actix_web::test::call_service(&app, request).await;
	assert_eq!(call_res.status(), actix_web::http::StatusCode::UNAUTHORIZED);

	let request = actix_web::test::TestRequest::put()
		.uri("/storage/gamma/documentG1.txt")
		.insert_header(("Authorization", format!("Bearer {token}")))
		.insert_header(("Content-Type", "text/plain"))
		.set_payload(b"documentG1-new".to_vec())
		.to_request();
	let call_res = actix_web::test::call_service(&app, request).await;
	assert_eq!(call_res.status(), actix_web::http::StatusCode::UNAUTHORIZED);
}
