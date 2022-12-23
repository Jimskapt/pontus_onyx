#[derive(Debug, PartialEq, Clone)]
pub struct LastModified(time::OffsetDateTime);

impl From<time::OffsetDateTime> for LastModified {
	fn from(data: time::OffsetDateTime) -> Self {
		Self(data)
	}
}
