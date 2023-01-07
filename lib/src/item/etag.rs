#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Etag(String);

impl<T: Into<String>> From<T> for Etag {
	fn from(new_value: T) -> Self {
		Self(new_value.into())
	}
}
impl std::fmt::Display for Etag {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		f.write_str(&self.0)
	}
}
