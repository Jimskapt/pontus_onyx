mod utils;

use pontus_onyx::Engine;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub async fn main() -> Result<(), JsValue> {
	#[cfg(debug_assertions)]
	utils::set_panic_hook();

	Ok(())
}

#[wasm_bindgen]
extern "C" {
	fn alert(s: &str);
	async fn invoke(name: &str, payload: JsValue) -> JsValue;
}

const PREFIX: &str = "pontus_onyx_wasm_example";

#[wasm_bindgen]
pub async fn load() {
	let window = web_sys::window().unwrap();
	let document = window.document().unwrap();

	let mut db =
		pontus_onyx::Database::new(pontus_onyx_engine_localstorage::LocalStorageEngine::new(
			pontus_onyx_engine_localstorage::EngineSettings {
				prefix: String::from(PREFIX),
			},
		));

	db.create_user("tfdrgdsfrtxgrdft", &mut String::from("chgbcfgyhg"), &[])
		.unwrap();

	let token = db
		.generate_token("tfdrgdsfrtxgrdft", &mut String::from("chgbcfgyhg"), "*:rw")
		.unwrap();

	let get = db
		.perform(
			pontus_onyx::Request::new(
				pontus_onyx::Method::Get,
				pontus_onyx::item::Path::try_from("/value.txt").unwrap(),
				"browser",
			)
			.token(token.clone()),
		)
		.await;

	if let pontus_onyx::ResponseStatus::Performed(
		pontus_onyx::EngineResponse::GetSuccessDocument(pontus_onyx::item::Item::Document {
			content: Some(content),
			..
		}),
	) = get.status
	{
		let content: String = content.try_into().unwrap();

		document
			.query_selector("#input")
			.unwrap()
			.unwrap()
			.dyn_into::<web_sys::HtmlInputElement>()
			.unwrap()
			.set_value(&content);

		let pre = document.create_element("pre").unwrap();
		pre.set_text_content(Some(&format!(
			"{} : successfully loaded value `{}` from the memory",
			time::OffsetDateTime::now_local().unwrap(),
			content
		)));

		document
			.query_selector("#logs")
			.unwrap()
			.unwrap()
			.append_child(&pre)
			.unwrap();
	} else {
		let pre = document.create_element("pre").unwrap();
		pre.set_text_content(Some(&format!(
			"{} : error while loading value : {:?}",
			time::OffsetDateTime::now_local().unwrap(),
			get.status
		)));

		document
			.query_selector("#logs")
			.unwrap()
			.unwrap()
			.append_child(&pre)
			.unwrap();
	}
}

#[wasm_bindgen]
pub async fn save() {
	let window = web_sys::window().unwrap();
	let document = window.document().unwrap();

	let new_value = document
		.query_selector("#input")
		.unwrap()
		.unwrap()
		.dyn_into::<web_sys::HtmlInputElement>()
		.unwrap()
		.value();

	let mut db =
		pontus_onyx::Database::new(pontus_onyx_engine_localstorage::LocalStorageEngine::new(
			pontus_onyx_engine_localstorage::EngineSettings {
				prefix: String::from(PREFIX),
			},
		));

	db.create_user("tfdrgdsfrtxgrdft", &mut String::from("chgbcfgyhg"), &[])
		.unwrap();

	let token = db
		.generate_token("tfdrgdsfrtxgrdft", &mut String::from("chgbcfgyhg"), "*:rw")
		.unwrap();

	let put = db
		.perform(
			pontus_onyx::Request::new(
				pontus_onyx::Method::Put,
				pontus_onyx::item::Path::try_from("/value.txt").unwrap(),
				"browser",
			)
			.token(token.clone())
			.item(
				pontus_onyx::item::Item::document()
					.content(&new_value)
					.content_type("text/plain"),
			),
		)
		.await;

	if let pontus_onyx::ResponseStatus::Performed(pontus_onyx::EngineResponse::CreateSuccess(
		_,
		_,
	)) = put.status
	{
		let pre = document.create_element("pre").unwrap();
		pre.set_text_content(Some(&format!(
			"{} : successfully save value `{}` for the first time",
			time::OffsetDateTime::now_local().unwrap(),
			new_value
		)));

		document
			.query_selector("#logs")
			.unwrap()
			.unwrap()
			.append_child(&pre)
			.unwrap();
	} else if let pontus_onyx::ResponseStatus::Performed(
		pontus_onyx::EngineResponse::UpdateSuccess(_, _),
	) = put.status
	{
		let pre = document.create_element("pre").unwrap();
		pre.set_text_content(Some(&format!(
			"{} : successfully replaced value in memory with `{}`",
			time::OffsetDateTime::now_local().unwrap(),
			new_value,
		)));

		document
			.query_selector("#logs")
			.unwrap()
			.unwrap()
			.append_child(&pre)
			.unwrap();
	} else {
		let pre = document.create_element("pre").unwrap();
		pre.set_text_content(Some(&format!(
			"{} : error while saving value : {:?}",
			time::OffsetDateTime::now_local().unwrap(),
			put.status
		)));

		document
			.query_selector("#logs")
			.unwrap()
			.unwrap()
			.append_child(&pre)
			.unwrap();
	}
}

#[wasm_bindgen]
pub async fn clear() {
	let window = web_sys::window().unwrap();
	let document = window.document().unwrap();

	let mut db =
		pontus_onyx::Database::new(pontus_onyx_engine_localstorage::LocalStorageEngine::new(
			pontus_onyx_engine_localstorage::EngineSettings {
				prefix: String::from(PREFIX),
			},
		));

	db.create_user("tfdrgdsfrtxgrdft", &mut String::from("chgbcfgyhg"), &[])
		.unwrap();

	let token = db
		.generate_token("tfdrgdsfrtxgrdft", &mut String::from("chgbcfgyhg"), "*:rw")
		.unwrap();

	let del = db
		.perform(
			pontus_onyx::Request::new(
				pontus_onyx::Method::Delete,
				pontus_onyx::item::Path::try_from("/value.txt").unwrap(),
				"browser",
			)
			.token(token.clone()),
		)
		.await;

	if let pontus_onyx::ResponseStatus::Performed(pontus_onyx::EngineResponse::DeleteSuccess) =
		del.status
	{
		let pre = document.create_element("pre").unwrap();
		pre.set_text_content(Some(&format!(
			"{} : successfully deleted value",
			time::OffsetDateTime::now_local().unwrap()
		)));

		document
			.query_selector("#logs")
			.unwrap()
			.unwrap()
			.append_child(&pre)
			.unwrap();

		document
			.query_selector("#input")
			.unwrap()
			.unwrap()
			.dyn_into::<web_sys::HtmlInputElement>()
			.unwrap()
			.set_value("");
	} else {
		let pre = document.create_element("pre").unwrap();
		pre.set_text_content(Some(&format!(
			"{} : error while deleting value : {:?}",
			time::OffsetDateTime::now_local().unwrap(),
			del.status
		)));

		document
			.query_selector("#logs")
			.unwrap()
			.unwrap()
			.append_child(&pre)
			.unwrap();
	}
}
