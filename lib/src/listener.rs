pub trait Listener {
	fn receive(&mut self, event: Event) -> crate::Response;
}

pub struct Event {
	pub id: String,
	pub date: crate::LastModified,
	pub method: EventMethod,
	pub path: crate::ItemPath,
	pub etag: crate::Etag,
	pub dbversion: String,
}
impl Event {
	pub fn build_from(request: &crate::Request, response: &crate::EngineResponse) -> Self {
		Self {
			id: format!("{}", uuid::Uuid::new_v4()),
			date: response
				.get_last_modified()
				.unwrap_or_else(|| crate::LastModified::from(time::OffsetDateTime::now_utc())),
			method: response.into(),
			path: request.path.clone(),
			etag: response.get_new_etag().unwrap(),
			dbversion: String::from(env!("CARGO_PKG_VERSION")),
		}
	}
}

pub enum EventMethod {
	Create,
	Update,
	Delete,
}

impl From<&crate::EngineResponse> for EventMethod {
	fn from(response: &crate::EngineResponse) -> Self {
		match response {
			crate::EngineResponse::GetSuccess(_) => panic!(),
			crate::EngineResponse::CreateSuccess(_, _) => EventMethod::Create,
			crate::EngineResponse::UpdateSuccess(_, _) => EventMethod::Update,
			crate::EngineResponse::ContentNotChanged => panic!(),
			crate::EngineResponse::DeleteSuccess(_) => EventMethod::Delete,
			crate::EngineResponse::NotFound => panic!(),
			crate::EngineResponse::NoIfMatch(_) => panic!(),
			crate::EngineResponse::IfNoneMatch(_) => panic!(),
			crate::EngineResponse::InternalError(_) => panic!(),
		}
	}
}
