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
	let temp: pontus_onyx::ItemPath = "folder_a/".try_into().unwrap();
	children.insert(temp.clone(), root.get(&temp).unwrap().clone());

	assert_eq!(
		tb.engine.perform(&Request::get("").unwrap()).await,
		EngineResponse::GetSuccessFolder {
			folder: root.get(&"".try_into().unwrap()).unwrap().clone(),
			children,
		},
	);
}
