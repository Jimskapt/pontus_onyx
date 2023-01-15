#[derive(derivative::Derivative, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[derivative(Debug = "transparent")]
pub struct Origin(pub String);

impl<T: Into<String>> From<T> for Origin {
	fn from(new_value: T) -> Self {
		Self(new_value.into())
	}
}
impl std::fmt::Display for Origin {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		f.write_str(&self.0)
	}
}
