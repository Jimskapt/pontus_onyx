#[derive(Debug, PartialEq)]
pub enum Limit {
	IfMatch(crate::Etag),
	IfNoneMatch(crate::Etag),
}

impl Limit {
	pub fn if_match(new_etag: impl Into<crate::Etag>) -> Self {
		Self::IfMatch(new_etag.into())
	}
	pub fn if_none_match(new_etag: impl Into<crate::Etag>) -> Self {
		Self::IfNoneMatch(new_etag.into())
	}
}
