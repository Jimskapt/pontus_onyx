use std::collections::BTreeMap;

// CUSTOM SETTINGS (please edit following) :
use crate::MemoryEngine as ThisEngine;

// GENERIC (please do NOT edit following) :
use pontus_onyx::{Engine, EngineResponse, Request};

fn build_toolbox() -> Toolbox {
	let engine = <ThisEngine as pontus_onyx::Engine>::new_for_tests();

	return Toolbox { engine };
}

struct Toolbox {
	engine: ThisEngine,
}

// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// //////////////////////////////////////////////// GET ////////////////////////////////////////////////////////////////
// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[tokio::test]
async fn get_empty_path() {
	let mut tb = build_toolbox();
	let root = tb.engine.root_for_tests().clone();

	let mut children = BTreeMap::new();
	let folder_a_path: pontus_onyx::ItemPath = "folder_a/".try_into().unwrap();
	let folder_b_path: pontus_onyx::ItemPath = "folder_b/".try_into().unwrap();
	let folder_public_path: pontus_onyx::ItemPath = "public/".try_into().unwrap();
	let root_doc_path: pontus_onyx::ItemPath = "document.txt".try_into().unwrap();
	children.insert(
		folder_a_path.clone(),
		root.get(&folder_a_path).unwrap().clone(),
	);
	children.insert(
		folder_b_path.clone(),
		root.get(&folder_b_path).unwrap().clone(),
	);
	children.insert(
		folder_public_path.clone(),
		root.get(&folder_public_path).unwrap().clone(),
	);
	children.insert(
		root_doc_path.clone(),
		root.get(&root_doc_path).unwrap().clone(),
	);

	assert_eq!(
		tb.engine
			.perform(&Request::get(pontus_onyx::ItemPath::try_from("").unwrap()))
			.await,
		EngineResponse::GetSuccessFolder {
			folder: root.get(&"".try_into().unwrap()).unwrap().clone(),
			children,
		},
	);
}

#[tokio::test]
async fn get_folder_a() {
	let mut tb = build_toolbox();
	let root = tb.engine.root_for_tests();

	let mut children = BTreeMap::new();
	let folder_a = pontus_onyx::ItemPath::try_from("folder_a/").unwrap();
	let folder_a_document = pontus_onyx::ItemPath::try_from("folder_a/document.txt").unwrap();
	children.insert(
		folder_a_document.clone(),
		root.get(&folder_a_document).unwrap().clone(),
	);

	assert_eq!(
		tb.engine.perform(&Request::get(&folder_a)).await,
		EngineResponse::GetSuccessFolder {
			folder: root.get(&folder_a).unwrap().clone(),
			children,
		},
	);
}

#[tokio::test]
async fn get_folder_a_document() {
	let mut tb = build_toolbox();
	let root = tb.engine.root_for_tests();

	let folder_a_document = pontus_onyx::ItemPath::try_from("folder_a/document.txt").unwrap();

	assert_eq!(
		tb.engine.perform(&Request::get(&folder_a_document)).await,
		EngineResponse::GetSuccessDocument(root.get(&folder_a_document).unwrap().clone()),
	);
}

#[tokio::test]
async fn get_folder_public() {
	let mut tb = build_toolbox();
	let root = tb.engine.root_for_tests();

	let mut children = BTreeMap::new();
	let folder_public = pontus_onyx::ItemPath::try_from("public/").unwrap();
	let folder_c = pontus_onyx::ItemPath::try_from("public/folder_c/").unwrap();
	children.insert(folder_c.clone(), root.get(&folder_c).unwrap().clone());

	assert_eq!(
		tb.engine.perform(&Request::get(&folder_public)).await,
		EngineResponse::GetSuccessFolder {
			folder: root.get(&folder_public).unwrap().clone(),
			children,
		},
	);
}

#[tokio::test]
async fn get_folder_public_document() {
	let mut tb = build_toolbox();
	let root = tb.engine.root_for_tests();

	let folder_public_document =
		pontus_onyx::ItemPath::try_from("public/folder_c/document.txt").unwrap();

	assert_eq!(
		tb.engine
			.perform(&Request::get(&folder_public_document))
			.await,
		EngineResponse::GetSuccessDocument(root.get(&folder_public_document).unwrap().clone()),
	);
}

#[tokio::test]
async fn get_not_existing_folder() {
	let mut tb = build_toolbox();

	let not_existing_path = pontus_onyx::ItemPath::try_from("not_existing/").unwrap();

	assert_eq!(
		tb.engine.perform(&Request::get(&not_existing_path)).await,
		EngineResponse::NotFound,
	);
}

#[tokio::test]
async fn get_not_existing_document() {
	let mut tb = build_toolbox();

	let not_existing_path = pontus_onyx::ItemPath::try_from("folder_a/not_existing.txt").unwrap();

	assert_eq!(
		tb.engine.perform(&Request::get(&not_existing_path)).await,
		EngineResponse::NotFound,
	);
}

#[tokio::test]
async fn get_not_existing_folder_and_document() {
	let mut tb = build_toolbox();

	let not_existing_path =
		pontus_onyx::ItemPath::try_from("not_existing/not_existing.txt").unwrap();

	assert_eq!(
		tb.engine.perform(&Request::get(&not_existing_path)).await,
		EngineResponse::NotFound,
	);
}

// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// //////////////////////////////////////////////// HEAD ///////////////////////////////////////////////////////////////
// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[tokio::test]
async fn head_folder_a() {
	let mut tb = build_toolbox();
	let root = tb.engine.root_for_tests();
	let folder_a = pontus_onyx::ItemPath::try_from("folder_a/").unwrap();

	assert_eq!(
		tb.engine.perform(&Request::head(&folder_a)).await,
		EngineResponse::GetSuccessFolder {
			folder: root.get(&folder_a).unwrap().clone(),
			children: BTreeMap::new(),
		},
	);
}

#[tokio::test]
async fn head_folder_a_document() {
	let mut tb = build_toolbox();
	let root = tb.engine.root_for_tests();

	let folder_a_document = pontus_onyx::ItemPath::try_from("folder_a/document.txt").unwrap();

	assert_eq!(
		tb.engine.perform(&Request::head(&folder_a_document)).await,
		EngineResponse::GetSuccessDocument(
			root.get(&folder_a_document)
				.unwrap()
				.clone_without_content()
		),
	);
}

#[tokio::test]
async fn head_not_existing_document() {
	let mut tb = build_toolbox();

	let not_existing_path = pontus_onyx::ItemPath::try_from("folder_a/not_existing.txt").unwrap();

	assert_eq!(
		tb.engine.perform(&Request::head(&not_existing_path)).await,
		EngineResponse::NotFound,
	);
}

// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// //////////////////////////////////////////////// PUT ////////////////////////////////////////////////////////////////
// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[tokio::test]
async fn put_new_document_in_existing_folder() {
	let mut tb = build_toolbox();

	let old_folder_data = {
		let root = tb.engine.root_for_tests();
		root.get(&pontus_onyx::ItemPath::try_from("folder_a/").unwrap())
			.unwrap()
			.clone()
	};

	let new_document_path = pontus_onyx::ItemPath::try_from("folder_a/new_document.txt").unwrap();

	let response = tb
		.engine
		.perform(
			&Request::put(&new_document_path).item(pontus_onyx::Item::Document {
				etag: None,
				last_modified: None,
				content: Some(b"new_document content".into()),
				content_type: Some("text/html".into()),
			}),
		)
		.await;

	dbg!(&response);
	assert!(response.has_muted_database());

	let new_folder_data = {
		let root = tb.engine.root_for_tests();
		root.get(&pontus_onyx::ItemPath::try_from("folder_a/").unwrap())
			.unwrap()
			.clone()
	};

	assert!(new_folder_data.get_etag().is_some());
	assert_eq!(response.get_new_etag(), new_folder_data.get_etag());
	assert_ne!(old_folder_data.get_etag(), new_folder_data.get_etag());

	assert!(new_folder_data.get_last_modified().is_some());
	assert_eq!(
		response.get_last_modified(),
		new_folder_data.get_last_modified()
	);
	assert_ne!(
		old_folder_data.get_last_modified(),
		new_folder_data.get_last_modified()
	);
	assert!(
		old_folder_data.get_last_modified().unwrap() < new_folder_data.get_last_modified().unwrap()
	);
}

#[tokio::test]
async fn put_new_document_in_new_folder() {
	let mut tb = build_toolbox();

	let new_document_path = pontus_onyx::ItemPath::try_from("new_folder/new_document.txt").unwrap();

	let response = tb
		.engine
		.perform(
			&Request::put(&new_document_path).item(pontus_onyx::Item::Document {
				etag: None,
				last_modified: None,
				content: Some(b"new_document content".into()),
				content_type: Some("text/html".into()),
			}),
		)
		.await;

	assert!(response.has_muted_database());

	let root = tb.engine.root_for_tests();
	assert!(root
		.get(&pontus_onyx::ItemPath::try_from("new_folder/").unwrap())
		.is_some());
	assert!(root
		.get(&pontus_onyx::ItemPath::try_from("new_folder/new_document.txt").unwrap())
		.is_some());
}
