#[derive(Debug, PartialEq)]
pub struct ContentType(String);

impl<T: Into<String>> From<T> for ContentType {
	fn from(new_value: T) -> Self {
		Self(new_value.into())
	}
}
