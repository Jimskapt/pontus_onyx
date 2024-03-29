use actix_web::HttpMessage;
use std::{
	io::Write,
	sync::{Arc, Mutex},
};

#[actix_rt::test]
async fn hsv5femo2qgu80gbad0ov5() {
	let settings = std::sync::Arc::new(std::sync::Mutex::new(crate::http_server::Settings::new(
		tempfile::tempdir().unwrap().into_path(),
	)));

	let logger = Arc::new(Mutex::new(charlie_buffalo::Logger::new(
		charlie_buffalo::new_dispatcher(Box::new(|_| {})),
		None,
	)));

	let mut app = actix_web::test::init_service(
		actix_web::App::new()
			.app_data(actix_web::web::Data::new(settings.clone()))
			.wrap(super::Auth { logger })
			.service(crate::http_server::get_favicon)
			.service(crate::http_server::options_favicon)
			.service(crate::http_server::api::get_item)
			.service(crate::http_server::webfinger_handle)
			.service(crate::http_server::get_oauth)
			.service(crate::http_server::post_oauth),
	)
	.await;

	let tests = vec![
		(010, "/storage/user/", true),
		(020, "/storage/user/folder/", true),
		(030, "/storage/user/document", true),
		(040, "/storage/user/folder/document", true),
		(050, "/storage/public/user/folder/", true),
		(060, "/storage/public/user/document", false),
		(070, "/storage/public/user/folder/document", false),
		(080, "/.well-known/webfinger", false),
		(090, "/oauth", false),
		(100, "/favicon.ico", false),
		(110, "/remotestorage.svg", false),
		(120, "/", false),
		(130, "/events/all", true),
	];

	for test in tests {
		print!("#{:03} : GET request to {} ... ", test.0, test.1);

		let request = actix_web::test::TestRequest::get().uri(test.1).to_request();
		let response = actix_web::test::call_service(&mut app, request).await;

		if test.2 {
			assert_eq!(response.status(), actix_web::http::StatusCode::UNAUTHORIZED);
		} else {
			assert_ne!(response.status(), actix_web::http::StatusCode::UNAUTHORIZED);
		}

		println!("OK");
	}
}

#[actix_rt::test]
async fn kp6m20xdwvw6v4t3yxq() {
	let access_tokens: std::sync::Arc<std::sync::Mutex<Vec<crate::http_server::AccessBearer>>> =
		std::sync::Arc::new(std::sync::Mutex::new(vec![]));

	let token = crate::http_server::AccessBearer::new(
		vec![
			crate::scope::Scope {
				module: String::from("folder_write"),
				right_type: crate::scope::ScopeRightType::ReadWrite,
			},
			crate::scope::Scope {
				module: String::from("folder_read"),
				right_type: crate::scope::ScopeRightType::Read,
			},
		],
		"test",
		"user",
	);
	access_tokens.lock().unwrap().push(token.clone());

	let database =
		crate::database::Database::new(Box::new(crate::database::sources::MemoryStorage {
			root_item: crate::item::Item::new_folder(vec![(
				"user",
				crate::item::Item::new_folder(vec![
					(
						"folder_write",
						crate::item::Item::new_folder(vec![(
							"a",
							crate::item::Item::Document {
								etag: crate::item::Etag::new(),
								content: Some(b"HELLO".to_vec()),
								content_type: crate::item::ContentType::from("text/plain"),
								last_modified: Some(time::OffsetDateTime::now_utc()),
							},
						)]),
					),
					(
						"folder_read",
						crate::item::Item::new_folder(vec![(
							"a",
							crate::item::Item::Document {
								etag: crate::item::Etag::new(),
								content: Some(b"HELLO".to_vec()),
								content_type: crate::item::ContentType::from("text/plain"),
								last_modified: Some(time::OffsetDateTime::now_utc()),
							},
						)]),
					),
					(
						"public",
						crate::item::Item::new_folder(vec![
							(
								"folder_write",
								crate::item::Item::new_folder(vec![(
									"a",
									crate::item::Item::Document {
										etag: crate::item::Etag::new(),
										content: Some(b"HELLO".to_vec()),
										content_type: crate::item::ContentType::from("text/plain"),
										last_modified: Some(time::OffsetDateTime::now_utc()),
									},
								)]),
							),
							(
								"folder_read",
								crate::item::Item::new_folder(vec![(
									"a",
									crate::item::Item::Document {
										etag: crate::item::Etag::new(),
										content: Some(b"HELLO".to_vec()),
										content_type: crate::item::ContentType::from("text/plain"),
										last_modified: Some(time::OffsetDateTime::now_utc()),
									},
								)]),
							),
						]),
					),
				]),
			)]),
		}));
	let database = std::sync::Arc::new(std::sync::Mutex::new(database));

	let settings = std::sync::Arc::new(std::sync::Mutex::new(crate::http_server::Settings::new(
		tempfile::tempdir().unwrap().into_path(),
	)));

	std::env::set_var("RUST_LOG", "debug");
	std::env::set_var("RUST_BACKTRACE", "1");
	env_logger::init();

	let logger = charlie_buffalo::Logger::new(
		charlie_buffalo::new_dispatcher(Box::from(move |log: charlie_buffalo::Log| {
			println!("{:?} : {:?}", log.attributes, log.content);
		})),
		charlie_buffalo::new_dropper(Box::from(|_: &charlie_buffalo::Logger| {})),
	);
	let logger = std::sync::Arc::new(std::sync::Mutex::new(logger));

	let working_folder = tempfile::tempdir().unwrap().into_path();

	let workspace_path_for_event_loop = working_folder.clone();
	let (history_sender, history_receiver) =
		std::sync::mpsc::channel::<crate::http_server::DbEvent>();

	std::thread::spawn(move || {
		let mut event_file = std::fs::File::options()
			.create(true)
			.append(true)
			.open(workspace_path_for_event_loop.join("events.bin"))
			.unwrap();

		loop {
			for event in &history_receiver {
				let mut row = serde_json::to_string(&event).unwrap();
				row += ",\n";
				event_file.write_all(row.as_bytes()).unwrap();
				event_file.flush().unwrap();
			}
		}
	});

	let mut app = actix_web::test::init_service(
		actix_web::App::new()
			.app_data(actix_web::web::Data::new(database.clone()))
			.app_data(actix_web::web::Data::new(access_tokens.clone()))
			.app_data(actix_web::web::Data::new(settings.clone()))
			.app_data(actix_web::web::Data::new(logger.clone()))
			.app_data(actix_web::web::Data::new(history_sender.clone()))
			.app_data(actix_web::web::Data::new(working_folder.clone()))
			.wrap(super::Auth {
				logger: logger.clone(),
			})
			.service(crate::http_server::api::get_item)
			.service(crate::http_server::api::put_item)
			.service(crate::http_server::server_events),
	)
	.await;

	let tests: Vec<(
		usize,
		actix_web::test::TestRequest,
		actix_web::http::StatusCode,
	)> = vec![
		(
			010,
			actix_web::test::TestRequest::get().uri("/storage/user/folder_read/"),
			actix_web::http::StatusCode::UNAUTHORIZED,
		),
		(
			020,
			actix_web::test::TestRequest::get().uri("/storage/user/folder_write/"),
			actix_web::http::StatusCode::UNAUTHORIZED,
		),
		(
			030,
			actix_web::test::TestRequest::get().uri("/storage/user/other/"),
			actix_web::http::StatusCode::UNAUTHORIZED,
		),
		(
			040,
			actix_web::test::TestRequest::get()
				.uri("/storage/user/folder_read/")
				.insert_header((
					actix_web::http::header::AUTHORIZATION,
					format!("Bearer {}", token.get_name()),
				)),
			actix_web::http::StatusCode::OK,
		),
		(
			050,
			actix_web::test::TestRequest::get()
				.uri("/storage/other_user/folder_read/")
				.insert_header((
					actix_web::http::header::AUTHORIZATION,
					format!("Bearer {}", token.get_name()),
				)),
			actix_web::http::StatusCode::FORBIDDEN,
		),
		(
			055,
			actix_web::test::TestRequest::get()
				.uri("/storage/user/should_not_be_accessed_by_this_token/")
				.insert_header((
					actix_web::http::header::AUTHORIZATION,
					format!("Bearer {}", token.get_name()),
				)),
			actix_web::http::StatusCode::FORBIDDEN,
		),
		(
			056,
			actix_web::test::TestRequest::get()
				.uri("/storage/user/should_not_be_accessed_by_this_token")
				.insert_header((
					actix_web::http::header::AUTHORIZATION,
					format!("Bearer {}", token.get_name()),
				)),
			actix_web::http::StatusCode::FORBIDDEN,
		),
		(
			060,
			actix_web::test::TestRequest::get()
				.uri("/storage/user/folder_write/")
				.insert_header((
					actix_web::http::header::AUTHORIZATION,
					format!("Bearer {}", token.get_name()),
				)),
			actix_web::http::StatusCode::OK,
		),
		(
			070,
			actix_web::test::TestRequest::get()
				.uri("/storage/user/should_not_be_accessed_by_this_token/")
				.insert_header((
					actix_web::http::header::AUTHORIZATION,
					format!("Bearer {}", token.get_name()),
				)),
			actix_web::http::StatusCode::FORBIDDEN,
		),
		(
			075,
			actix_web::test::TestRequest::get()
				.uri("/storage/user/should_not_be_accessed_by_this_token")
				.insert_header((
					actix_web::http::header::AUTHORIZATION,
					format!("Bearer {}", token.get_name()),
				)),
			actix_web::http::StatusCode::FORBIDDEN,
		),
		(
			080,
			actix_web::test::TestRequest::get()
				.uri("/storage/user/folder_read/")
				.insert_header((
					actix_web::http::header::AUTHORIZATION,
					format!("Bearer {}", "RANDOM_BEARER"),
				)),
			actix_web::http::StatusCode::UNAUTHORIZED,
		),
		(
			090,
			actix_web::test::TestRequest::get()
				.uri("/storage/user/folder_write/")
				.insert_header((
					actix_web::http::header::AUTHORIZATION,
					format!("Bearer {}", "RANDOM_BEARER"),
				)),
			actix_web::http::StatusCode::UNAUTHORIZED,
		),
		(
			100,
			actix_web::test::TestRequest::get()
				.uri("/storage/user/other/")
				.insert_header((
					actix_web::http::header::AUTHORIZATION,
					format!("Bearer {}", "RANDOM_BEARER"),
				)),
			actix_web::http::StatusCode::UNAUTHORIZED,
		),
		(
			110,
			actix_web::test::TestRequest::put()
				.uri("/storage/user/folder_read/b")
				.insert_header((
					actix_web::http::header::AUTHORIZATION,
					format!("Bearer {}", token.get_name()),
				))
				.set_json(&serde_json::json!({"value": "HELLO"})),
			actix_web::http::StatusCode::FORBIDDEN,
		),
		(
			120,
			actix_web::test::TestRequest::put()
				.uri("/storage/user/folder_write/b")
				.insert_header((
					actix_web::http::header::AUTHORIZATION,
					format!("Bearer {}", token.get_name()),
				))
				.set_json(&serde_json::json!({"value": "HELLO"})),
			actix_web::http::StatusCode::CREATED,
		),
		(
			130,
			actix_web::test::TestRequest::put()
				.uri("/storage/other_user/folder_write/b")
				.insert_header((
					actix_web::http::header::AUTHORIZATION,
					format!("Bearer {}", token.get_name()),
				))
				.set_json(&serde_json::json!({"value": "HELLO"})),
			actix_web::http::StatusCode::FORBIDDEN,
		),
		(
			140,
			actix_web::test::TestRequest::put()
				.uri("/storage/user/other/b")
				.insert_header((
					actix_web::http::header::AUTHORIZATION,
					format!("Bearer {}", token.get_name()),
				))
				.set_json(&serde_json::json!({"value": "HELLO"})),
			actix_web::http::StatusCode::FORBIDDEN,
		),
		(
			150,
			actix_web::test::TestRequest::put()
				.uri("/storage/public/user/folder_read/b")
				.insert_header((
					actix_web::http::header::AUTHORIZATION,
					format!("Bearer {}", token.get_name()),
				))
				.set_json(&serde_json::json!({"value": "HELLO"})),
			actix_web::http::StatusCode::FORBIDDEN,
		),
		(
			160,
			actix_web::test::TestRequest::put()
				.uri("/storage/public/user/folder_write/b")
				.insert_header((
					actix_web::http::header::AUTHORIZATION,
					format!("Bearer {}", token.get_name()),
				))
				.set_json(&serde_json::json!({"value": "HELLO"})),
			actix_web::http::StatusCode::CREATED,
		),
		(
			170,
			actix_web::test::TestRequest::put()
				.uri("/storage/public/user/other/b")
				.insert_header((
					actix_web::http::header::AUTHORIZATION,
					format!("Bearer {}", token.get_name()),
				))
				.set_json(&serde_json::json!({"value": "HELLO"})),
			actix_web::http::StatusCode::FORBIDDEN,
		),
		(
			180,
			actix_web::test::TestRequest::put()
				.uri("/storage/user/folder_read/b")
				.insert_header((
					actix_web::http::header::AUTHORIZATION,
					format!("Bearer {}", "RANDOM_BEARER"),
				))
				.set_json(&serde_json::json!({"value": "HELLO"})),
			actix_web::http::StatusCode::UNAUTHORIZED,
		),
		(
			190,
			actix_web::test::TestRequest::put()
				.uri("/storage/user/folder_write/b")
				.insert_header((
					actix_web::http::header::AUTHORIZATION,
					format!("Bearer {}", "RANDOM_BEARER"),
				))
				.set_json(&serde_json::json!({"value": "HELLO"})),
			actix_web::http::StatusCode::UNAUTHORIZED,
		),
		(
			200,
			actix_web::test::TestRequest::put()
				.uri("/storage/user/other/b")
				.insert_header((
					actix_web::http::header::AUTHORIZATION,
					format!("Bearer {}", "RANDOM_BEARER"),
				))
				.set_json(&serde_json::json!({"value": "HELLO"})),
			actix_web::http::StatusCode::UNAUTHORIZED,
		),
		(
			210,
			actix_web::test::TestRequest::get()
				.uri("/events/all")
				.insert_header((
					actix_web::http::header::AUTHORIZATION,
					format!("Bearer {}", token.get_name()),
				)),
			actix_web::http::StatusCode::OK,
		),
		(
			220,
			actix_web::test::TestRequest::get()
				.uri("/events/all")
				.insert_header((
					actix_web::http::header::AUTHORIZATION,
					format!("Bearer {}", "RANDOM_BEARER"),
				)),
			actix_web::http::StatusCode::UNAUTHORIZED,
		),
		(
			230,
			actix_web::test::TestRequest::get().uri("/events/all"),
			actix_web::http::StatusCode::UNAUTHORIZED,
		),
	];

	for test in tests {
		let request = test.1.to_request();
		print!(
			"#{:03} : {} request to {} with Authorization = {:?} ... ",
			test.0,
			request.method(),
			request.path(),
			match request
				.headers()
				.iter()
				.find(|&(name, _)| name == actix_web::http::header::AUTHORIZATION)
			{
				Some((_, value)) => format!("{}[...]", &value.to_str().unwrap()[7..7 + 10]),
				None => String::from("None"),
			}
		);

		let response = actix_web::test::call_service(&mut app, request).await;

		assert_eq!(response.status(), test.2);

		println!("OK");
	}
}
