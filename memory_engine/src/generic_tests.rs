// CUSTOM SETTINGS (edit following) :
use crate::MemoryEngine as ThisEngine;

// GENERIC (do not edit following) :
use pontus_onyx::{Engine, EngineResponse, Request};

fn build_toolbox() -> Toolbox {
	let engine = <ThisEngine as pontus_onyx::Engine>::new_for_tests();

	return Toolbox { engine };
}

struct Toolbox {
	engine: ThisEngine,
}

#[test]
fn get_empty_path() {
	let mut tb = build_toolbox();
	let root = tb.engine.root_for_tests();

	assert_eq!(
		tb.engine.perform(&Request::get("").unwrap()),
		EngineResponse::GetSuccess(root),
	);
}
