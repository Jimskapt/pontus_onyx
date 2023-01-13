#[actix_web::main]
async fn main() -> std::io::Result<()> {
	env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

	let mut storage_db = pontus_onyx::Database::new(pontus_onyx_memory_engine::MemoryEngine::new());

	storage_db.create_user("toto", &mut String::from("toto"));
	let token = storage_db
		.generate_token("toto", &mut String::from("toto"), "*:rw")
		.unwrap();

	log::debug!("debug token = {token:?}");

	let storage_db = std::sync::Arc::new(std::sync::Mutex::new(storage_db));

	actix_web::HttpServer::new(move || {
		actix_web::App::new()
			.app_data(actix_web::web::Data::new(storage_db.clone()))
			.route("/storage/{path:.*}", actix_web::web::head().to(storage))
			.route("/storage/{path:.*}", actix_web::web::get().to(storage))
			.route("/storage/{path:.*}", actix_web::web::put().to(storage))
			.route("/storage/{path:.*}", actix_web::web::delete().to(storage))
	})
	.bind(("127.0.0.1", 8080))?
	.run()
	.await
}

async fn storage(
	storage_db: actix_web::web::Data<
		std::sync::Arc<
			std::sync::Mutex<pontus_onyx::Database<pontus_onyx_memory_engine::MemoryEngine>>,
		>,
	>,
	request: actix_web::HttpRequest,
	payload: actix_web::web::Payload,
) -> actix_web::HttpResponse {
	let converted_request = pontus_onyx::from_actix_request(&request, &mut payload.into_inner())
		.await
		.unwrap();

	let db_response = storage_db.lock().unwrap().perform(converted_request).await;

	actix_web::HttpResponse::from(db_response)
}
