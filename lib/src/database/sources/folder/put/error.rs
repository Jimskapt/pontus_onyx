#[derive(Debug, PartialEq, Eq)]
pub enum PutError {
	GetError(super::super::GetError),
	DoesNotWorksForFolders,
	ContentNotChanged,
	CanNotReadFile {
		os_path: std::path::PathBuf,
		error: String,
	},
	CanNotWriteFile {
		os_path: std::path::PathBuf,
		error: String,
	},
	CanNotSerializeFile {
		os_path: std::path::PathBuf,
		error: String,
	},
	CanNotDeserializeFile {
		os_path: std::path::PathBuf,
		error: String,
	},
}
impl std::fmt::Display for PutError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::GetError(get_error) => std::fmt::Display::fmt(get_error, f),
			Self::DoesNotWorksForFolders => {
				f.write_str("this method does not works for folders in payload")
			}
			Self::ContentNotChanged => f.write_str("the content has not changed"),
			Self::CanNotReadFile { os_path, error } => f.write_fmt(format_args!(
				"can not read file `{:?}` because : {}",
				os_path, error
			)),
			Self::CanNotWriteFile { os_path, error } => f.write_fmt(format_args!(
				"can not write file `{:?}` because : {}",
				os_path, error
			)),
			Self::CanNotSerializeFile { os_path, error } => f.write_fmt(format_args!(
				"can not serialize file `{:?}` because : {}",
				os_path, error
			)),
			Self::CanNotDeserializeFile { os_path, error } => f.write_fmt(format_args!(
				"can not deserialize file `{:?}` because : {}",
				os_path, error
			)),
		}
	}
}
impl std::error::Error for PutError {}
#[cfg(feature = "server")]
impl crate::database::Error for PutError {
	fn to_response(&self, origin: &str, should_have_body: bool) -> actix_web::HttpResponse {
		match self {
			// TODO : we have to find a way to change method
			Self::GetError(get_error) => {
				crate::database::Error::to_response(get_error, origin, should_have_body)
			}
			Self::DoesNotWorksForFolders => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::BAD_REQUEST,
				None,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::ContentNotChanged => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::NOT_MODIFIED,
				None,
				None,
				Some(format!("{}", self)),
				should_have_body,
			),
			Self::CanNotReadFile {
				os_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				None,
				should_have_body,
			),
			Self::CanNotWriteFile {
				os_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				None,
				should_have_body,
			),
			Self::CanNotSerializeFile {
				os_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				None,
				should_have_body,
			),
			Self::CanNotDeserializeFile {
				os_path: _,
				error: _,
			} => crate::database::build_http_json_response(
				origin,
				&actix_web::http::Method::PUT,
				actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
				None,
				None,
				None,
				should_have_body,
			),
		}
	}
}
