#[derive(Debug, PartialEq, Clone, PartialOrd, Ord, Eq)]
pub struct ItemPath(Vec<ItemPathPart>);

impl ItemPath {
	pub fn starts_with<P: TryInto<ItemPath, Error = ItemPathConvertError> + std::fmt::Debug>(
		&self,
		start: P,
	) -> bool {
		let start_debug = format!("{start:?}");

		match start.try_into() {
			Ok(ItemPath(start)) => self
				.0
				.iter()
				.take(start.len())
				.collect::<Vec<&ItemPathPart>>()
				.eq(&start.iter().collect::<Vec<&ItemPathPart>>()),
			Err(err) => {
				log::warn!("incorrect start value `{start_debug}` : {err:?}");
				false
			}
		}
	}
}

#[test]
fn start_with_incorrect_value() {
	assert_eq!(
		ItemPath(vec![
			ItemPathPart::Folder(String::from("path")),
			ItemPathPart::Folder(String::from("to")),
			ItemPathPart::Folder(String::from("f\0lder")),
		])
		.starts_with("path/to/f\0lder"),
		false
	);
}

#[test]
fn start_with_correct_folder_value() {
	assert_eq!(
		ItemPath(vec![
			ItemPathPart::Folder(String::from("public")),
			ItemPathPart::Folder(String::from("path")),
			ItemPathPart::Folder(String::from("to")),
			ItemPathPart::Document(String::from("document")),
		])
		.starts_with("public/"),
		true
	);
}

#[test]
fn start_with_correct_full_folder_value() {
	assert_eq!(
		ItemPath(vec![
			ItemPathPart::Folder(String::from("public")),
			ItemPathPart::Folder(String::from("path")),
			ItemPathPart::Folder(String::from("to")),
			ItemPathPart::Document(String::from("document")),
		])
		.starts_with("public/path/to/document"),
		true
	);
}

#[test]
fn start_with_correct_document_value() {
	assert_eq!(
		ItemPath(vec![
			ItemPathPart::Folder(String::from("public")),
			ItemPathPart::Folder(String::from("path")),
			ItemPathPart::Folder(String::from("to")),
			ItemPathPart::Document(String::from("document")),
		])
		.starts_with("public"),
		false
	);
}

impl TryFrom<&str> for ItemPath {
	type Error = ItemPathConvertError;

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
					Ok(item_name) => result.push(ItemPathPart::Folder(item_name)),
					Err(error) => {
						return Err(ItemPathConvertError::WrongItemPartName {
							until: ItemPath(result),
							error,
						});
					}
				}
			}

			if !last_is_folder {
				if let Some(last) = result.last() {
					let name = String::from(last.get_name());
					*result.last_mut().unwrap() = ItemPathPart::Document(name);
				}
			}

			Ok(Self(result))
		} else {
			Ok(Self(vec![]))
		}
	}
}
impl TryFrom<String> for ItemPath {
	type Error = ItemPathConvertError;

	fn try_from(input: String) -> Result<Self, Self::Error> {
		Self::try_from(input.as_str())
	}
}
impl TryFrom<&String> for ItemPath {
	type Error = ItemPathConvertError;

	fn try_from(input: &String) -> Result<Self, Self::Error> {
		Self::try_from(input.as_str())
	}
}

#[test]
fn try_from_prefix() {
	assert_eq!(
		ItemPath::try_from("/path/to/document").unwrap(),
		ItemPath::try_from("/path/to/document").unwrap()
	);
}

#[test]
fn try_from_document() {
	assert_eq!(
		ItemPath::try_from("/path/to/document").unwrap(),
		ItemPath(vec![
			ItemPathPart::Folder(String::from("path")),
			ItemPathPart::Folder(String::from("to")),
			ItemPathPart::Document(String::from("document")),
		]),
	);
}

#[test]
fn try_from_folder() {
	assert_eq!(
		ItemPath::try_from("/path/to/folder/").unwrap(),
		ItemPath(vec![
			ItemPathPart::Folder(String::from("path")),
			ItemPathPart::Folder(String::from("to")),
			ItemPathPart::Folder(String::from("folder")),
		]),
	);
}

#[test]
fn try_from_empty() {
	assert_eq!(ItemPath::try_from("").unwrap(), ItemPath(vec![]),);
}

#[test]
fn try_from_space_empty() {
	assert_eq!(ItemPath::try_from(" ").unwrap(), ItemPath(vec![]),);
}

#[test]
fn try_from_empty_name() {
	assert_eq!(
		ItemPath::try_from("/path/to//document"),
		Err(ItemPathConvertError::WrongItemPartName {
			until: ItemPath::try_from("/path/to/").unwrap(),
			error: ItemPathPartConvertError::IsEmpty,
		}),
	);
}

#[test]
fn try_from_single_point_name() {
	assert_eq!(
		ItemPath::try_from("/path/to/./document"),
		Err(ItemPathConvertError::WrongItemPartName {
			until: ItemPath::try_from("/path/to/").unwrap(),
			error: ItemPathPartConvertError::IsSinglePoint,
		}),
	);
}

#[test]
fn try_from_double_point_name() {
	assert_eq!(
		ItemPath::try_from("/path/to/../document"),
		Err(ItemPathConvertError::WrongItemPartName {
			until: ItemPath::try_from("/path/to/").unwrap(),
			error: ItemPathPartConvertError::IsDoublePoint,
		}),
	);
}

#[derive(Debug, PartialEq)]
pub enum ItemPathConvertError {
	WrongItemPartName {
		until: ItemPath,
		error: ItemPathPartConvertError,
	},
}

impl ItemPath {
	pub fn target_is_document(&self) -> bool {
		matches!(self.0.last(), Some(ItemPathPart::Document(_)))
	}
}

#[derive(Debug, PartialEq, Clone, PartialOrd, Ord, Eq)]
pub enum ItemPathPart {
	Folder(String),
	Document(String),
}
impl ItemPathPart {
	fn get_name(&self) -> &str {
		match self {
			Self::Folder(name) => name,
			Self::Document(name) => name,
		}
	}
}

fn check_item_path_part_name(input: impl Into<String>) -> Result<String, ItemPathPartConvertError> {
	let input = input.into();

	if input.is_empty() {
		Err(ItemPathPartConvertError::IsEmpty)
	} else if input == "." {
		Err(ItemPathPartConvertError::IsSinglePoint)
	} else if input == ".." {
		Err(ItemPathPartConvertError::IsDoublePoint)
	} else if input.contains('/') {
		Err(ItemPathPartConvertError::ContainsSlash)
	} else if input.contains('\\') {
		Err(ItemPathPartConvertError::ContainsBackslash)
	} else if input.contains('\0') {
		Err(ItemPathPartConvertError::ContainsZero)
	} else if input.contains(".itemdata.") {
		Err(ItemPathPartConvertError::ContainsItemData)
	} else {
		Ok(input)
	}
}

#[derive(Debug, PartialEq)]
pub enum ItemPathPartConvertError {
	IsEmpty,
	IsSinglePoint,
	IsDoublePoint,
	ContainsSlash,
	ContainsBackslash,
	ContainsZero,
	ContainsItemData,
}
