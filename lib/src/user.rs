use std::collections::BTreeMap;

#[derive(zeroize::Zeroize, zeroize::ZeroizeOnDrop, serde::Serialize, serde::Deserialize)]
pub struct User {
	#[zeroize(skip)]
	pub username: String,
	pub password: String,
	#[zeroize(skip)]
	pub tokens: BTreeMap<crate::security::Token, crate::security::TokenMetadata>,
}

impl serde_encrypt::traits::SerdeEncryptSharedKey for User {
	type S = serde_encrypt::serialize::impls::BincodeSerializer<Self>;
}
