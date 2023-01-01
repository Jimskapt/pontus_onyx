#[derive(Debug, PartialEq)]
pub enum Limit {
	IfMatch(crate::item::Etag),
	IfNoneMatch(crate::item::Etag),
}

impl Limit {
	pub fn if_match(new_etag: impl Into<crate::item::Etag>) -> Self {
		Self::IfMatch(new_etag.into())
	}
	pub fn if_none_match(new_etag: impl Into<crate::item::Etag>) -> Self {
		Self::IfNoneMatch(new_etag.into())
	}
}
