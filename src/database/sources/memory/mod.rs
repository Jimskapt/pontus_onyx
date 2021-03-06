mod delete;
mod get;
mod put;

pub use delete::DeleteError;
pub use get::GetError;
pub use put::PutError;

/// Store data only in R.A.M.
///
/// Warning, all data disappears when this source is dropped from memory !
///
/// This storage is useful in context without other storage or ephemeral systems,
/// like sandboxes without filesystem or unit tests, for example.
#[derive(Debug)]
pub struct MemoryStorage {
	/// All data is stored inside `content` of this item, so it should be only the [`Folder`][`crate::item::Item::Folder`] variant.
	pub root_item: crate::item::Item,
}
impl crate::database::DataSource for MemoryStorage {
	fn get(
		&self,
		path: &crate::item::ItemPath,
		if_match: &crate::item::Etag,
		if_none_match: &[&crate::item::Etag],
		_get_content: bool,
	) -> Result<crate::item::Item, Box<dyn std::error::Error>> {
		get::get(&self.root_item, path, if_match, if_none_match)
	}

	fn put(
		&mut self,
		path: &crate::item::ItemPath,
		if_match: &crate::item::Etag,
		if_none_match: &[&crate::item::Etag],
		new_item: crate::item::Item,
	) -> crate::database::PutResult {
		put::put(&mut self.root_item, path, if_match, if_none_match, new_item)
	}

	fn delete(
		&mut self,
		path: &crate::item::ItemPath,
		if_match: &crate::item::Etag,
	) -> Result<crate::item::Etag, Box<dyn std::error::Error>> {
		delete::delete(&mut self.root_item, path, if_match)
	}
}
