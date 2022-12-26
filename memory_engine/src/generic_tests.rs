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

#[tokio::test]
async fn get_empty_path() {
	let mut tb = build_toolbox();
	let root = tb.engine.root_for_tests();

	let mut children = BTreeMap::new();
	let folder_a_path: pontus_onyx::ItemPath = "folder_a/".try_into().unwrap();
	children.insert(folder_a_path.clone(), root.get(&folder_a_path).unwrap().clone());

	assert_eq!(
		tb.engine.perform(&Request::get(pontus_onyx::ItemPath::try_from("").unwrap())).await,
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
	children.insert(folder_a_document.clone(), root.get(&folder_a_document).unwrap().clone());

	assert_eq!(
		tb.engine.perform(&Request::get(&folder_a)).await,
		EngineResponse::GetSuccessFolder {
			folder: root.get(&folder_a).unwrap().clone(),
			children,
		},
	);
}
