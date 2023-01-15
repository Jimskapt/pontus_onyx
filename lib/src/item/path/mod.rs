#[cfg(test)]
pub mod tests;

#[derive(
	derivative::Derivative,
	PartialEq,
	Clone,
	PartialOrd,
	Ord,
	Eq,
	serde::Serialize,
	serde::Deserialize,
)]
#[derivative(Debug = "transparent")]
pub struct Path(Vec<PathPart>);

impl Path {
	pub fn starts_with(&self, start: &Self) -> bool {
		self.0
			.iter()
			.take(start.0.len())
			.collect::<Vec<&PathPart>>()
			.eq(&start.0.iter().collect::<Vec<&PathPart>>())
	}
	pub fn is_folder(&self) -> bool {
		if !self.0.is_empty() {
			matches!(self.0.last(), Some(PathPart::Folder(_)))
		} else {
			true
		}
	}
	pub fn is_document(&self) -> bool {
		matches!(self.0.last(), Some(PathPart::Document(_)))
	}
	pub fn is_direct_child(&self, parent: &Self) -> bool {
		self.starts_with(parent) && self.0.len() == parent.0.len() + 1
	}
	pub fn parent(&self) -> Option<Self> {
		let len = self.0.len();
		if len > 1 {
			Some(Self(self.0.iter().take(len - 1).cloned().collect()))
		} else {
			None
		}
	}
	pub fn last(&self) -> Option<&PathPart> {
		self.0.last()
	}
}
impl std::fmt::Display for Path {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		let mut result = String::new();

		for part in &self.0 {
			result += &match part {
				PathPart::Folder(name) => {
					format!("{name}/")
				}
				PathPart::Document(name) => name.clone(),
			}
		}

		f.write_str(&result)
	}
}

pub static ROOT_PATH: Path = Path(vec![]);

impl TryFrom<&str> for Path {
	type Error = PathConvertError;

	fn try_from(input: &str) -> Result<Self, Self::Error> {
		let mut last_is_folder = false;
		let input = input.strip_prefix('/').unwrap_or(input);
		let input = match input.strip_suffix('/') {
			Some(input) => {
				last_is_folder = true;
				input
			}
			None => input,
		};
		let input = input.trim();

		if !input.is_empty() {
			let mut result = vec![];

			for item_name in input.split('/') {
				match check_item_path_part_name(item_name) {
					Ok(item_name) => result.push(PathPart::Folder(item_name)),
					Err(error) => {
						return Err(PathConvertError::WrongItemPartName {
							until: Path(result),
							error,
						});
					}
				}
			}

			if !last_is_folder {
				if let Some(last) = result.last() {
					let name = String::from(last.get_name());
					*result.last_mut().unwrap() = PathPart::Document(name);
				}
			}

			Ok(Self(result))
		} else {
			Ok(Self(vec![]))
		}
	}
}
impl TryFrom<String> for Path {
	type Error = PathConvertError;

	fn try_from(input: String) -> Result<Self, Self::Error> {
		Self::try_from(input.as_str())
	}
}
impl TryFrom<&String> for Path {
	type Error = PathConvertError;

	fn try_from(input: &String) -> Result<Self, Self::Error> {
		Self::try_from(input.as_str())
	}
}
impl AsRef<Path> for Path {
	fn as_ref(&self) -> &Self {
		self
	}
}

#[derive(Debug, PartialEq)]
pub enum PathConvertError {
	WrongItemPartName {
		until: Path,
		error: PathPartConvertError,
	},
}
impl std::fmt::Display for PathConvertError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::WrongItemPartName { until, error } => f.write_fmt(format_args!(
				"wrong item part name until `{until}` : {error}"
			)),
		}
	}
}
impl std::error::Error for PathConvertError {}

#[derive(Debug, PartialEq, Clone, PartialOrd, Ord, Eq, serde::Serialize, serde::Deserialize)]
pub enum PathPart {
	Folder(String),
	Document(String),
}
impl PathPart {
	fn get_name(&self) -> &str {
		match self {
			Self::Folder(name) => name,
			Self::Document(name) => name,
		}
	}
}
impl std::fmt::Display for PathPart {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::Document(name) => f.write_str(name),
			Self::Folder(name) => f.write_fmt(format_args!("{}/", name)),
		}
	}
}

fn check_item_path_part_name(input: impl Into<String>) -> Result<String, PathPartConvertError> {
	let input = input.into();

	if input.is_empty() {
		Err(PathPartConvertError::IsEmpty)
	} else if input == "." {
		Err(PathPartConvertError::IsSinglePoint)
	} else if input == ".." {
		Err(PathPartConvertError::IsDoublePoint)
	} else if input.contains('/') {
		Err(PathPartConvertError::ContainsSlash)
	} else if input.contains('\\') {
		Err(PathPartConvertError::ContainsBackslash)
	} else if input.contains('\0') {
		Err(PathPartConvertError::ContainsZero)
	} else if input.contains(".itemdata.") {
		Err(PathPartConvertError::ContainsItemData)
	} else {
		Ok(input)
	}
}

#[derive(Debug, PartialEq)]
pub enum PathPartConvertError {
	IsEmpty,
	IsSinglePoint,
	IsDoublePoint,
	ContainsSlash,
	ContainsBackslash,
	ContainsZero,
	ContainsItemData,
}
impl std::fmt::Display for PathPartConvertError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::IsEmpty => f.write_fmt(format_args!("name is empty")),
			Self::IsSinglePoint => f.write_fmt(format_args!("name is only a point (`.`)")),
			Self::IsDoublePoint => f.write_fmt(format_args!("name is only a double-point (`..`)")),
			Self::ContainsSlash => f.write_fmt(format_args!("name contains a slash (`/`)")),
			Self::ContainsBackslash => {
				f.write_fmt(format_args!("name contains a backslash (`\\`)"))
			}
			Self::ContainsZero => f.write_fmt(format_args!("name contains the empty char (`\\0`)")),
			Self::ContainsItemData => {
				f.write_fmt(format_args!("name contains the chain `.itemdata.`"))
			}
		}
	}
}
