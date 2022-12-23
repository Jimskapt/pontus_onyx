#[derive(Default)]
pub struct MemoryEngine {}

impl pontus_onyx::Engine for MemoryEngine {
	fn perform(&mut self, request: &pontus_onyx::Request) -> pontus_onyx::EngineResponse {
		pontus_onyx::EngineResponse::InternalError(format!("TODO"))
	}

	fn new_for_tests() -> Self {
		Self {}
	}
	fn root_for_tests(&self) -> pontus_onyx::Item {
		todo!()
	}
}

#[cfg(test)]
mod generic_tests;
