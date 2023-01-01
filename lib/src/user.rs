use std::collections::BTreeMap;

#[derive(zeroize::Zeroize, zeroize::ZeroizeOnDrop)]
pub struct User {
	#[zeroize(skip)]
	pub username: String,
	pub password: String,
	#[zeroize(skip)]
	pub tokens: BTreeMap<crate::security::Token, crate::security::TokenMetadata>,
}
