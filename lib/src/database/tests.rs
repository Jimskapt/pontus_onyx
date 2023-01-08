use std::collections::BTreeMap;

use crate::{item::Path, AccessError, Database, Engine, EngineResponse, Request, ResponseStatus};

#[test]
fn generate_token() {
	let mut database = Database::new(<EmptyEngineForTests as Engine>::new_for_tests());
	database.create_user("my_user", &mut String::from("my_password"));

	assert!(database
		.generate_token("", &mut String::from("my_password"), "my_access:rw")
		.is_err());
	assert!(database
		.generate_token(
			"WRONG_user",
			&mut String::from("my_password"),
			"my_access:rw"
		)
		.is_err());

	assert!(database
		.generate_token(
			"my_user",
			&mut String::from("WRONG_password"),
			"my_access:rw"
		)
		.is_err());
	assert!(database
		.generate_token("my_user", &mut String::from(""), "my_access:rw")
		.is_err());

	assert!(database
		.generate_token(
			"my_user",
			&mut String::from("my_password"),
			"WRONG/ACCESS:rw"
		)
		.is_err());
	assert!(database
		.generate_token("my_user", &mut String::from("my_password"), "")
		.is_err());

	assert!(database
		.generate_token(
			"WRONG_user",
			&mut String::from("WRONG_password"),
			"WRONG/ACCESS:rw"
		)
		.is_err());

	assert!(database
		.generate_token("my_user", &mut String::from("my_password"), "my_access:rw")
		.is_ok());
	assert!(!database
		.generate_token("my_user", &mut String::from("my_password"), "my_access:rw")
		.unwrap()
		.0
		.is_empty());
}

#[tokio::test]
async fn should_not_list_public() {
	assert_eq!(
		Database::new(<EmptyEngineForTests as Engine>::new_for_tests())
			.perform(Request::get(Path::try_from("public/").unwrap()))
			.await
			.status,
		ResponseStatus::Unauthorized(AccessError::CanNotListPublic)
	);
}

#[tokio::test]
async fn should_not_list_public_subfolder() {
	assert_eq!(
		Database::new(<EmptyEngineForTests as Engine>::new_for_tests())
			.perform(Request::get(Path::try_from("public/folder/").unwrap()))
			.await
			.status,
		ResponseStatus::Unauthorized(AccessError::CanNotListPublic)
	);
}

#[tokio::test]
async fn should_pass_public_get() {
	assert_eq!(
		Database::new(<EmptyEngineForTests as Engine>::new_for_tests())
			.perform(Request::get(Path::try_from("public/document.txt").unwrap()))
			.await
			.status,
		ResponseStatus::Performed(EngineResponse::InternalError(String::from(
			EMPTY_ENGINE_PASS_RESPONSE
		)))
	);
}

#[tokio::test]
async fn should_pass_public_get_subfolder() {
	assert_eq!(
		Database::new(<EmptyEngineForTests as Engine>::new_for_tests())
			.perform(Request::get(
				Path::try_from("public/folder/document.txt").unwrap()
			))
			.await
			.status,
		ResponseStatus::Performed(EngineResponse::InternalError(String::from(
			EMPTY_ENGINE_PASS_RESPONSE
		)))
	);
}

#[tokio::test]
async fn should_not_pass_without_token() {
	assert_eq!(
		Database::new(<EmptyEngineForTests as Engine>::new_for_tests())
			.perform(Request::get(Path::try_from("folder_a/").unwrap()))
			.await
			.status,
		ResponseStatus::Unauthorized(AccessError::MissingToken)
	);
}

#[tokio::test]
async fn should_pass_with_right_token() {
	let mut database = Database::new(<EmptyEngineForTests as Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:r")
		.unwrap();

	assert_eq!(
		database
			.perform(Request::get(Path::try_from("folder_a/").unwrap()).token(token))
			.await
			.status,
		ResponseStatus::Performed(EngineResponse::InternalError(String::from(
			EMPTY_ENGINE_PASS_RESPONSE
		)))
	);
}

#[tokio::test]
async fn should_not_pass_with_wrong_token() {
	let mut database = Database::new(<EmptyEngineForTests as Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_b:r")
		.unwrap();

	assert_eq!(
		database
			.perform(Request::get(Path::try_from("folder_a/").unwrap()).token(token))
			.await
			.status,
		ResponseStatus::Unauthorized(crate::AccessError::NotValidToken(vec![
			crate::security::TokenValidityError::RequestError(
				crate::security::RequestValidityError::OutOfModuleScope
			)
		]))
	);
}

#[tokio::test]
async fn should_not_pass_with_token_but_wrong_method() {
	let mut database = Database::new(<EmptyEngineForTests as Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:r")
		.unwrap();

	assert_eq!(
		database
			.perform(
				Request::put(Path::try_from("folder_a/document.json").unwrap())
					.token(token)
					.item(crate::item::Item::Document {
						etag: None,
						last_modified: None,
						content: Some(br#"{"key": "value"}"#.into()),
						content_type: Some("application/json".into())
					})
			)
			.await
			.status,
		ResponseStatus::Unauthorized(crate::AccessError::NotValidToken(vec![
			crate::security::TokenValidityError::RequestError(
				crate::security::RequestValidityError::UnallowedMethod
			)
		]))
	);
}

#[tokio::test]
async fn get_no_if_match() {
	let mut database = Database::new(<EmptyEngineForTests as Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:r")
		.unwrap();

	assert_eq!(
		database
			.perform(
				Request::get(Path::try_from("folder_a/existing.json").unwrap())
					.token(token)
					.add_limit(crate::Limit::IfMatch("WRONG_ETAG".into()))
			)
			.await
			.status,
		ResponseStatus::NoIfMatch("DOCUMENT_ETAG".into())
	);
}

#[tokio::test]
async fn get_if_match() {
	let mut database = Database::new(<EmptyEngineForTests as Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:r")
		.unwrap();

	assert_eq!(
		database
			.perform(
				Request::get(Path::try_from("folder_a/existing.json").unwrap())
					.token(token)
					.add_limit(crate::Limit::IfMatch("DOCUMENT_ETAG".into()))
			)
			.await
			.status,
		ResponseStatus::Performed(EngineResponse::GetSuccessDocument(
			crate::item::Item::Document {
				etag: Some("DOCUMENT_ETAG".into()),
				last_modified: Some(
					time::OffsetDateTime::from_unix_timestamp(1000)
						.unwrap()
						.into()
				),
				content: Some(b"DOCUMENT_CONTENT".into()),
				content_type: Some("DOCUMENT_CONTENT_TYPE".into())
			}
		))
	);
}

#[tokio::test]
async fn get_if_none_match() {
	let mut database = Database::new(<EmptyEngineForTests as Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:r")
		.unwrap();

	assert_eq!(
		database
			.perform(
				Request::get(Path::try_from("folder_a/existing.json").unwrap())
					.token(token)
					.add_limit(crate::Limit::IfNoneMatch("DOCUMENT_ETAG".into()))
			)
			.await
			.status,
		ResponseStatus::IfNoneMatch("DOCUMENT_ETAG".into())
	);
}

#[tokio::test]
async fn get_no_if_none_match() {
	let mut database = Database::new(<EmptyEngineForTests as Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:r")
		.unwrap();

	assert_eq!(
		database
			.perform(
				Request::get(Path::try_from("folder_a/existing.json").unwrap())
					.token(token)
					.add_limit(crate::Limit::IfNoneMatch("ANOTHER_DOCUMENT_ETAG".into()))
			)
			.await
			.status,
		ResponseStatus::Performed(EngineResponse::GetSuccessDocument(
			crate::item::Item::Document {
				etag: Some("DOCUMENT_ETAG".into()),
				last_modified: Some(
					time::OffsetDateTime::from_unix_timestamp(1000)
						.unwrap()
						.into()
				),
				content: Some(b"DOCUMENT_CONTENT".into()),
				content_type: Some("DOCUMENT_CONTENT_TYPE".into())
			}
		))
	);
}

#[tokio::test]
async fn get_if_none_match_all() {
	let mut database = Database::new(<EmptyEngineForTests as Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:r")
		.unwrap();

	assert_eq!(
		database
			.perform(
				Request::get(Path::try_from("folder_a/existing.json").unwrap())
					.token(token)
					.add_limit(crate::Limit::IfNoneMatch("*".into()))
			)
			.await
			.status,
		ResponseStatus::IfNoneMatch("DOCUMENT_ETAG".into())
	);
}

#[tokio::test]
async fn put_content_not_changed() {
	let mut database = Database::new(<EmptyEngineForTests as Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:rw")
		.unwrap();

	assert_eq!(
		database
			.perform(
				Request::put(Path::try_from("folder_a/existing.json").unwrap())
					.token(token)
					.item(crate::item::Item::Document {
						etag: None,
						last_modified: None,
						content: Some("DOCUMENT_CONTENT".into()),
						content_type: Some("DOCUMENT_CONTENT_TYPE".into())
					})
			)
			.await
			.status,
		ResponseStatus::ContentNotChanged
	);
}

#[tokio::test]
async fn put_folder_path() {
	let mut database = Database::new(<EmptyEngineForTests as Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:rw")
		.unwrap();

	assert_eq!(
		database
			.perform(
				Request::put(Path::try_from("folder_a/folder_aa/").unwrap())
					.token(token)
					.item(crate::item::Item::Document {
						etag: None,
						last_modified: None,
						content: Some("DOCUMENT_CONTENT".into()),
						content_type: Some("DOCUMENT_CONTENT_TYPE".into())
					})
			)
			.await
			.status,
		ResponseStatus::NotSuitableForFolderItem
	);
}

#[tokio::test]
async fn put_folder_item() {
	let mut database = Database::new(<EmptyEngineForTests as Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:rw")
		.unwrap();

	assert_eq!(
		database
			.perform(
				Request::put(Path::try_from("folder_a/folder_aa").unwrap())
					.token(token)
					.item(crate::item::Item::Folder {
						etag: None,
						last_modified: None,
					})
			)
			.await
			.status,
		ResponseStatus::NotSuitableForFolderItem
	);
}

#[tokio::test]
async fn put_none_item() {
	let mut database = Database::new(<EmptyEngineForTests as Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:rw")
		.unwrap();

	assert_eq!(
		database
			.perform(Request::put(Path::try_from("folder_a/document.txt").unwrap()).token(token))
			.await
			.status,
		ResponseStatus::MissingRequestItem
	);
}

#[tokio::test]
async fn get_not_found() {
	let mut database = Database::new(<EmptyEngineForTests as Engine>::new_for_tests());
	database.create_user("username", &mut String::from("password"));
	let token = database
		.generate_token("username", &mut String::from("password"), "folder_a:r")
		.unwrap();

	assert_eq!(
		database
			.perform(
				Request::get(Path::try_from("folder_a/not_existing.json").unwrap())
					.token(token)
					.item(crate::item::Item::Document {
						etag: None,
						last_modified: None,
						content: Some("DOCUMENT_CONTENT".into()),
						content_type: Some("DOCUMENT_CONTENT_TYPE".into())
					})
			)
			.await
			.status,
		ResponseStatus::Performed(EngineResponse::NotFound)
	);
}

static EMPTY_ENGINE_PASS_RESPONSE: &str = "TEST ENGINE : NOT EXPECTED FOR PRODUCTION USE";

struct EmptyEngineForTests {}

#[async_trait::async_trait]
impl Engine for EmptyEngineForTests {
	async fn perform(&mut self, request: &Request) -> EngineResponse {
		if request.path == "folder_a/existing.json".try_into().unwrap() {
			return EngineResponse::GetSuccessDocument(crate::item::Item::Document {
				etag: Some("DOCUMENT_ETAG".into()),
				last_modified: Some(
					time::OffsetDateTime::from_unix_timestamp(1000)
						.unwrap()
						.into(),
				),
				content: Some(b"DOCUMENT_CONTENT".into()),
				content_type: Some("DOCUMENT_CONTENT_TYPE".into()),
			});
		} else if request.path == "folder_a/not_existing.json".try_into().unwrap() {
			return EngineResponse::NotFound;
		}

		return EngineResponse::InternalError(String::from(EMPTY_ENGINE_PASS_RESPONSE));
	}

	fn new_for_tests() -> Self {
		Self {}
	}
	fn root_for_tests(&self) -> BTreeMap<Path, crate::item::Item> {
		BTreeMap::new()
	}
}
