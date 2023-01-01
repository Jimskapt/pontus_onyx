#[derive(Debug, PartialEq, Clone)]
pub struct Content(Vec<u8>);

impl<T: AsRef<[u8]>> From<T> for Content {
	fn from(data: T) -> Self {
		Self(data.as_ref().to_vec())
	}
}
