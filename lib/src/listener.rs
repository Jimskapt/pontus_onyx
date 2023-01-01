use crate::EngineResponse;

pub trait Listener {
	fn receive(&mut self, event: Event) -> crate::Response;
}

pub struct Event {
	pub id: String,
	pub date: crate::item::LastModified,
	pub method: EventMethod,
	pub path: crate::item::Path,
	pub etag: crate::item::Etag,
	pub dbversion: String,
}
impl Event {
	pub fn build_from(request: &crate::Request, response: &EngineResponse) -> Result<Self, ()> {
		EventMethod::try_from(response).map(|method| Self {
			id: format!("{}", uuid::Uuid::new_v4()),
			date: response.get_last_modified().unwrap_or_else(|| {
				crate::item::LastModified::from(time::OffsetDateTime::now_utc())
			}),
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

impl TryFrom<&EngineResponse> for EventMethod {
	type Error = ();

	fn try_from(response: &EngineResponse) -> Result<Self, Self::Error> {
		match response {
			EngineResponse::GetSuccessDocument(_) => Err(()),
			EngineResponse::GetSuccessFolder { .. } => Err(()),
			EngineResponse::CreateSuccess(_, _) => Ok(EventMethod::Create),
			EngineResponse::UpdateSuccess(_, _) => Ok(EventMethod::Update),
			EngineResponse::DeleteSuccess => Ok(EventMethod::Delete),
			EngineResponse::NotFound => Err(()),
			EngineResponse::InternalError(_) => Err(()),
		}
	}
}
