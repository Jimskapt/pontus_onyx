#[derive(Debug, PartialEq, Clone)]
pub struct Etag(String);

impl<T: Into<String>> From<T> for Etag {
	fn from(new_value: T) -> Self {
		Self(new_value.into())
	}
}
