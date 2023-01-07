#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord, serde::Serialize, serde::Deserialize)]
pub struct LastModified(time::OffsetDateTime);

impl LastModified {
	pub fn into_inner(&self) -> &time::OffsetDateTime {
		&self.0
	}
}

impl From<time::OffsetDateTime> for LastModified {
	fn from(data: time::OffsetDateTime) -> Self {
		Self(data)
	}
}
