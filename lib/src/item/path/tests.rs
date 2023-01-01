use super::{Path, PathConvertError, PathPart, PathPartConvertError, ROOT_PATH};

#[test]
fn start_with_correct_folder_value() {
	assert_eq!(
		Path(vec![
			PathPart::Folder(String::from("public")),
			PathPart::Folder(String::from("path")),
			PathPart::Folder(String::from("to")),
			PathPart::Document(String::from("document")),
		])
		.starts_with(&Path(vec![PathPart::Folder(String::from("public"))])),
		true
	);
}

#[test]
fn start_with_correct_full_folder_value() {
	assert_eq!(
		Path(vec![
			PathPart::Folder(String::from("public")),
			PathPart::Folder(String::from("path")),
			PathPart::Folder(String::from("to")),
			PathPart::Document(String::from("document")),
		])
		.starts_with(&Path(vec![
			PathPart::Folder(String::from("public")),
			PathPart::Folder(String::from("path")),
			PathPart::Folder(String::from("to")),
			PathPart::Document(String::from("document")),
		])),
		true
	);
}

#[test]
fn start_with_correct_document_value() {
	assert_eq!(
		Path(vec![
			PathPart::Folder(String::from("public")),
			PathPart::Folder(String::from("path")),
			PathPart::Folder(String::from("to")),
			PathPart::Document(String::from("document")),
		])
		.starts_with(&Path(vec![
			PathPart::Folder(String::from("public")),
			PathPart::Folder(String::from("path")),
			PathPart::Folder(String::from("to")),
			PathPart::Document(String::from("document")),
		])),
		true
	);
}

#[test]
fn is_direct_child() {
	assert_eq!(
		Path(vec![
			PathPart::Folder(String::from("public")),
			PathPart::Folder(String::from("path")),
			PathPart::Folder(String::from("to")),
			PathPart::Document(String::from("document")),
		])
		.is_direct_child(&Path(vec![
			PathPart::Folder(String::from("public")),
			PathPart::Folder(String::from("path")),
			PathPart::Folder(String::from("to")),
		])),
		true
	)
}

#[test]
fn is_not_direct_child() {
	assert_eq!(
		Path(vec![
			PathPart::Folder(String::from("public")),
			PathPart::Folder(String::from("path")),
			PathPart::Folder(String::from("to")),
			PathPart::Document(String::from("document")),
		])
		.is_direct_child(&Path(vec![
			PathPart::Folder(String::from("public")),
			PathPart::Folder(String::from("path")),
		])),
		false
	)
}

#[test]
fn is_not_direct_child_no_common() {
	assert_eq!(
		Path(vec![
			PathPart::Folder(String::from("public")),
			PathPart::Folder(String::from("path")),
			PathPart::Folder(String::from("to")),
			PathPart::Document(String::from("document")),
		])
		.is_direct_child(&Path(vec![
			PathPart::Folder(String::from("no")),
			PathPart::Folder(String::from("common")),
		])),
		false
	)
}

#[test]
fn parent_3() {
	assert_eq!(
		Path(vec![
			PathPart::Folder(String::from("public")),
			PathPart::Folder(String::from("path")),
			PathPart::Folder(String::from("to")),
		])
		.parent(),
		Some(Path(vec![
			PathPart::Folder(String::from("public")),
			PathPart::Folder(String::from("path")),
		]))
	);
}

#[test]
fn parent_2() {
	assert_eq!(
		Path(vec![
			PathPart::Folder(String::from("public")),
			PathPart::Folder(String::from("path")),
		])
		.parent(),
		Some(Path(vec![PathPart::Folder(String::from("public")),]))
	);
}

#[test]
fn parent_1() {
	assert_eq!(
		Path(vec![PathPart::Folder(String::from("public")),]).parent(),
		None
	);
}

/////////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn try_from_prefix() {
	assert_eq!(
		Path::try_from("/path/to/document"),
		Ok(Path(vec![
			PathPart::Folder(String::from("path")),
			PathPart::Folder(String::from("to")),
			PathPart::Document(String::from("document")),
		]))
	);
}

#[test]
fn try_from_document() {
	assert_eq!(
		Path::try_from("/path/to/document"),
		Ok(Path(vec![
			PathPart::Folder(String::from("path")),
			PathPart::Folder(String::from("to")),
			PathPart::Document(String::from("document")),
		])),
	);
}

#[test]
fn try_from_folder() {
	assert_eq!(
		Path::try_from("/path/to/folder/"),
		Ok(Path(vec![
			PathPart::Folder(String::from("path")),
			PathPart::Folder(String::from("to")),
			PathPart::Folder(String::from("folder")),
		])),
	);
}

#[test]
fn try_from_empty() {
	assert_eq!(Path::try_from(""), Ok(Path(vec![])));
	assert_eq!(Path::try_from(""), Ok(ROOT_PATH.clone()));
}

#[test]
fn try_from_space_empty() {
	assert_eq!(Path::try_from(" "), Ok(Path(vec![])));
	assert_eq!(Path::try_from(" "), Ok(ROOT_PATH.clone()));
}

#[test]
fn try_from_empty_name() {
	assert_eq!(
		Path::try_from("/path/to//document"),
		Err(PathConvertError::WrongItemPartName {
			until: Path::try_from("/path/to/").unwrap(),
			error: PathPartConvertError::IsEmpty,
		}),
	);
}

#[test]
fn try_from_single_point_name() {
	assert_eq!(
		Path::try_from("/path/to/./document"),
		Err(PathConvertError::WrongItemPartName {
			until: Path::try_from("/path/to/").unwrap(),
			error: PathPartConvertError::IsSinglePoint,
		}),
	);
}

#[test]
fn try_from_double_point_name() {
	assert_eq!(
		Path::try_from("/path/to/../document"),
		Err(PathConvertError::WrongItemPartName {
			until: Path::try_from("/path/to/").unwrap(),
			error: PathPartConvertError::IsDoublePoint,
		}),
	);
}
