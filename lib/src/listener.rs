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
	pub fn build_from(
		request: &crate::Request,
		response: &crate::EngineResponse,
	) -> Result<Self, ()> {
		EventMethod::try_from(response).map(|method| Self {
			id: format!("{}", uuid::Uuid::new_v4()),
			date: response
				.get_last_modified()
				.unwrap_or_else(|| crate::LastModified::from(time::OffsetDateTime::now_utc())),
			method,
			path: request.path.clone(),
			etag: response.get_new_etag().unwrap(),
			dbversion: String::from(env!("CARGO_PKG_VERSION")),
		})
	}
}

pub enum EventMethod {
	Create,
	Update,
	Delete,
}

impl TryFrom<&crate::EngineResponse> for EventMethod {
	type Error = ();

	fn try_from(response: &crate::EngineResponse) -> Result<Self, Self::Error> {
		match response {
			crate::EngineResponse::GetSuccessDocument(_) => Err(()),
			crate::EngineResponse::GetSuccessFolder { .. } => Err(()),
			crate::EngineResponse::CreateSuccess(_, _) => Ok(EventMethod::Create),
			crate::EngineResponse::UpdateSuccess(_, _) => Ok(EventMethod::Update),
			crate::EngineResponse::DeleteSuccess => Ok(EventMethod::Delete),
			crate::EngineResponse::NotFound => Err(()),
			crate::EngineResponse::InternalError(_) => Err(()),
		}
	}
}
