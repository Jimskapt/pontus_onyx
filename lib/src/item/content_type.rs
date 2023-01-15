#[derive(derivative::Derivative, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[derivative(Debug = "transparent")]
pub struct ContentType(String);

impl ContentType {
	pub fn into_inner(&self) -> &str {
		&self.0
	}
}

impl<T: Into<String>> From<T> for ContentType {
	fn from(new_value: T) -> Self {
		Self(new_value.into())
	}
}
