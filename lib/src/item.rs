#[derive(Debug, PartialEq)]
pub enum Item {
	Document {
		etag: Option<crate::Etag>,
		last_modified: Option<crate::LastModified>,
		content: Option<crate::Content>,
		content_type: Option<crate::ContentType>,
	},
	Folder {
		etag: Option<crate::Etag>,
		last_modified: Option<crate::LastModified>,
	},
}

impl Item {
	pub fn document() -> Self {
		Self::Document {
			etag: None,
			last_modified: None,
			content: None,
			content_type: None,
		}
	}
	pub fn content(mut self, new_content: impl Into<crate::Content>) -> Self {
		if let Self::Document {
			ref mut content, ..
		} = self
		{
			content.replace(new_content.into());
		} else {
			log::warn!("can not replace content on item which is not document");
		}

		return self;
	}
	pub fn content_type(mut self, new_content_type: impl Into<crate::ContentType>) -> Self {
		if let Self::Document {
			ref mut content_type,
			..
		} = self
		{
			content_type.replace(new_content_type.into());
		} else {
			log::warn!("can not replace content_type on item which is not document");
		}

		return self;
	}
}

#[cfg(test)]
impl Clone for Item {
	fn clone(&self) -> Self {
		todo!()
	}
}
