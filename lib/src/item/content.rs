#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Content(Vec<u8>);

impl Content {
	pub fn into_inner(&self) -> &[u8] {
		&self.0
	}
}

impl<T: AsRef<[u8]>> From<T> for Content {
	fn from(data: T) -> Self {
		Self(data.as_ref().to_vec())
	}
}
