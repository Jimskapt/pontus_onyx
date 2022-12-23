#[derive(Debug, PartialEq, Clone, PartialOrd, Ord, Eq)]
pub struct Token(pub String);

impl<T: Into<String>> From<T> for Token {
	fn from(value: T) -> Self {
		Self(value.into())
	}
}
