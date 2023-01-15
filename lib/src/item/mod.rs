mod content;
mod content_type;
mod etag;
mod last_modified;
mod path;

pub use content::*;
pub use content_type::*;
pub use etag::*;
pub use last_modified::*;
pub use path::{Path, PathConvertError, ROOT_PATH};

#[derive(derivative::Derivative, PartialEq, Clone)]
#[derivative(Debug)]
pub enum Item {
	Document {
		etag: Option<Etag>,
		last_modified: Option<LastModified>,
		#[derivative(Debug(format_with = "hidden_content"))]
		content: Option<Content>,
		content_type: Option<ContentType>,
	},
	Folder {
		etag: Option<Etag>,
		last_modified: Option<LastModified>,
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
	pub fn content(mut self, new_content: impl Into<Content>) -> Self {
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
	pub fn content_type(mut self, new_content_type: impl Into<ContentType>) -> Self {
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
	pub fn clone_without_content(&self) -> Self {
		match self {
			Self::Folder { .. } => self.clone(),
			Self::Document {
				etag,
				last_modified,
				content: _,
				content_type,
			} => Self::Document {
				etag: etag.clone(),
				last_modified: last_modified.clone(),
				content: None,
				content_type: content_type.clone(),
			},
		}
	}
}

impl Item {
	pub fn get_etag(&self) -> Option<Etag> {
		match self {
			Self::Document { etag, .. } => etag.clone(),
			Self::Folder { etag, .. } => etag.clone(),
		}
	}
	pub fn get_last_modified(&self) -> Option<LastModified> {
		match self {
			Self::Document { last_modified, .. } => last_modified.clone(),
			Self::Folder { last_modified, .. } => last_modified.clone(),
		}
	}
}

fn hidden_content(_: &Option<Content>, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
	f.write_str("[hidden]")
}
